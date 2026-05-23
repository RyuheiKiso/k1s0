#!/usr/bin/env python3
"""
hlc_drift_verifier.py — k1s0 cross-axis HLC ドリフト検証器

製造業プラットフォーム k1s0 の複数軸（tier1/tier2/tier3/infra/data/security/ops/client）
が発行する HLC タイムスタンプの因果順序違反およびクロック間ドリフトを検出する。

検出する問題の種類:
  1. 因果順序違反（causal_violation）:
     イベント A が論理的に B の原因であるにも関わらず、HLC で A の順序が B より後になっている。
     例: tier1 が tier2 にリクエストを送信（A）し、tier2 が処理した（B）が、
         A の hlc_wall_ms > B の hlc_wall_ms となっている場合。

  2. 壁時計ドリフト（wall_clock_drift）:
     同一時刻に発行されたはずのイベント間で wall_ms の差が閾値（デフォルト 5000ms）を超えている。

  3. 単調性違反（monotonicity_violation）:
     同一ノード内で tick の順序が逆転している（logical カウンタが減少している）。

使用例:
  collector = EventCollector(log_dir=Path("./logs"))
  events = collector.collect(axes=["tier1", "tier2", "tier3"])
  result = verify_cross_axis_ordering(events, max_drift_ms=5000)
  if result.has_violations:
      for v in result.violations:
          print(v)
"""

from __future__ import annotations

# dataclasses: 検証結果の値オブジェクトを定義するために使用する
import dataclasses
# enum: 違反の種類を列挙型で表現するために使用する
import enum
# logging: 検証過程のデバッグ情報を出力するために使用する
import logging
# typing: 型ヒントに使用する
from typing import Dict, List, Optional, Sequence, Tuple

# 同一パッケージの event_collector から AxisEvent をインポートする
from .event_collector import AxisEvent

# ロガーを初期化する（モジュール名をロガー名として使用する）
logger = logging.getLogger(__name__)

# デフォルトの最大許容ドリフト（ミリ秒）を定義する
DEFAULT_MAX_DRIFT_MS: int = 5_000

# 製造ラインのリアルタイム制御用の厳格な最大許容ドリフト（ミリ秒）を定義する
STRICT_MAX_DRIFT_MS: int = 500


class ViolationType(enum.Enum):
    """HLC ドリフト/違反の種類を表す列挙型。"""

    # 因果順序違反: A→B の因果関係があるが HLC で A >= B になっている
    CAUSAL_VIOLATION = "causal_violation"
    # 壁時計ドリフト: 同期的なイベント間の wall_ms 差が閾値を超えている
    WALL_CLOCK_DRIFT = "wall_clock_drift"
    # 単調性違反: 同一ノード内で HLC の順序が逆転している
    MONOTONICITY_VIOLATION = "monotonicity_violation"
    # 孤立イベント: 他軸との相関を持たない孤立したイベント（情報警告）
    ISOLATED_EVENT = "isolated_event"


@dataclasses.dataclass(frozen=True)
class DriftViolation:
    """
    単一のドリフト違反を記録する不変データクラス。

    violation_type によって意味が変わるフィールドの説明:
      CAUSAL_VIOLATION:
        axis_a/event_a  : 原因イベント（先に発生したはず）
        axis_b/event_b  : 結果イベント（後に発生したはず）
        delta_ms        : A.wall_ms - B.wall_ms（正値 = A の方が遅い → 違反）

      WALL_CLOCK_DRIFT:
        axis_a/event_a  : 比較基準イベント
        axis_b/event_b  : ドリフトが検出されたイベント
        delta_ms        : |A.wall_ms - B.wall_ms|（閾値を超えた差分）

      MONOTONICITY_VIOLATION:
        axis_a          : 違反が発生したノードの軸名
        event_a         : 前のイベント
        event_b         : 後のイベント（HLC が逆転している）
        delta_ms        : 逆転量（絶対値）
    """

    # violation_type: 違反の種類（ViolationType 列挙型）
    violation_type: ViolationType
    # axis_a: 比較元の軸名
    axis_a: str
    # event_a: 比較元のイベント ID
    event_a: str
    # axis_b: 比較先の軸名
    axis_b: str
    # event_b: 比較先のイベント ID
    event_b: str
    # hlc_a: 比較元の HLC タイムスタンプ (wall_ms, logical, node_id)
    hlc_a: Tuple[int, int, int]
    # hlc_b: 比較先の HLC タイムスタンプ (wall_ms, logical, node_id)
    hlc_b: Tuple[int, int, int]
    # delta_ms: 違反の大きさ（wall_ms の差分、ミリ秒単位）
    delta_ms: int
    # description: 人間が読める違反の説明文
    description: str

    def __str__(self) -> str:
        """違反を人間が読める文字列に変換する。"""
        # 違反の種類、軸ペア、デルタ、説明を含む文字列を構築する
        return (
            f"[{self.violation_type.value}] "
            f"{self.axis_a}/{self.event_a} <-> {self.axis_b}/{self.event_b} "
            f"delta={self.delta_ms}ms: {self.description}"
        )


@dataclasses.dataclass
class HLCDriftResult:
    """
    verify_cross_axis_ordering() の検証結果を格納するデータクラス。

    has_violations が True の場合は violations リストを確認すること。
    """

    # violations: 検出されたすべての違反リスト
    violations: List[DriftViolation]
    # max_observed_drift_ms: 検出された最大ドリフト（ミリ秒）
    max_observed_drift_ms: int
    # axis_pairs_checked: 検証した軸ペアの数
    axis_pairs_checked: int
    # total_events_analyzed: 分析したイベントの総数
    total_events_analyzed: int
    # axes_present: 入力に存在した軸の集合
    axes_present: List[str]

    @property
    def has_violations(self) -> bool:
        """violations リストが空でない場合に True を返す。"""
        # violations が空かどうかを確認する
        return len(self.violations) > 0

    @property
    def causal_violations(self) -> List[DriftViolation]:
        """CAUSAL_VIOLATION 型の違反のみを抽出して返す。"""
        # 因果順序違反のみをフィルタリングして返す
        return [v for v in self.violations if v.violation_type == ViolationType.CAUSAL_VIOLATION]

    @property
    def drift_violations(self) -> List[DriftViolation]:
        """WALL_CLOCK_DRIFT 型の違反のみを抽出して返す。"""
        # 壁時計ドリフト違反のみをフィルタリングして返す
        return [v for v in self.violations if v.violation_type == ViolationType.WALL_CLOCK_DRIFT]

    @property
    def monotonicity_violations(self) -> List[DriftViolation]:
        """MONOTONICITY_VIOLATION 型の違反のみを抽出して返す。"""
        # 単調性違反のみをフィルタリングして返す
        return [v for v in self.violations if v.violation_type == ViolationType.MONOTONICITY_VIOLATION]

    def summary(self) -> str:
        """検証結果のサマリ文字列を返す。"""
        # 総違反数、種類別の内訳、最大ドリフトを含むサマリを構築する
        lines = [
            f"HLCDriftResult: {len(self.violations)} violations",
            f"  causal_violations      : {len(self.causal_violations)}",
            f"  drift_violations       : {len(self.drift_violations)}",
            f"  monotonicity_violations: {len(self.monotonicity_violations)}",
            f"  max_observed_drift_ms  : {self.max_observed_drift_ms}",
            f"  axis_pairs_checked     : {self.axis_pairs_checked}",
            f"  total_events_analyzed  : {self.total_events_analyzed}",
            f"  axes_present           : {', '.join(sorted(self.axes_present))}",
        ]
        # 改行で結合して返す
        return "\n".join(lines)


def _hlc_key(event: AxisEvent) -> Tuple[int, int, int]:
    """AxisEvent の HLC キータプルを返す（比較用）。"""
    # (hlc_wall_ms, hlc_logical, hlc_node_id) の辞書順でキーを返す
    return (event.hlc_wall_ms, event.hlc_logical, event.hlc_node_id)


def _check_monotonicity(
    events_by_node: Dict[Tuple[str, int], List[AxisEvent]],
    violations: List[DriftViolation],
) -> int:
    """
    同一ノード内のイベント列が HLC 単調増加であることを検証する。

    違反が検出された場合は violations リストに追加する。
    戻り値は検出された違反の数。
    """
    # 違反カウンタを初期化する
    violation_count = 0
    # 各ノード（軸名 + node_id のペア）について単調性を確認する
    for (axis_name, node_id), node_events in events_by_node.items():
        # node_events が 2 件未満の場合は比較できないのでスキップする
        if len(node_events) < 2:
            continue
        # 連続するイベントペアを比較する
        for i in range(1, len(node_events)):
            # 前のイベントを取得する
            prev = node_events[i - 1]
            # 現在のイベントを取得する
            curr = node_events[i]
            # HLC キーを比較する（前 <= 現在 が正常）
            if _hlc_key(curr) < _hlc_key(prev):
                # 単調性違反を記録する
                delta = prev.hlc_wall_ms - curr.hlc_wall_ms
                violation = DriftViolation(
                    violation_type=ViolationType.MONOTONICITY_VIOLATION,
                    axis_a=axis_name,
                    event_a=prev.event_id,
                    axis_b=axis_name,
                    event_b=curr.event_id,
                    hlc_a=_hlc_key(prev),
                    hlc_b=_hlc_key(curr),
                    delta_ms=abs(delta),
                    description=(
                        f"ノード {axis_name}:{node_id} で HLC が逆転しています: "
                        f"{prev.event_id} ({_hlc_key(prev)}) > {curr.event_id} ({_hlc_key(curr)})"
                    ),
                )
                # 違反を violations リストに追加する
                violations.append(violation)
                # 違反カウンタを増加させる
                violation_count += 1
    # 検出した違反の数を返す
    return violation_count


def _check_wall_clock_drift(
    events: Sequence[AxisEvent],
    max_drift_ms: int,
    violations: List[DriftViolation],
) -> int:
    """
    隣接するイベント間の wall_ms 差が max_drift_ms を超えないことを検証する。

    連続するイベントペアを比較し、wall_ms の差が閾値を超えた場合に違反を記録する。
    戻り値は検出された最大ドリフト（ミリ秒）。
    """
    # 最大観測ドリフトを初期化する
    max_observed = 0
    # イベントが 2 件未満の場合は比較できないのでスキップする
    if len(events) < 2:
        return 0
    # 連続するイベントペアを比較する
    for i in range(1, len(events)):
        # 前のイベントを取得する
        prev = events[i - 1]
        # 現在のイベントを取得する
        curr = events[i]
        # wall_ms の差分を計算する（絶対値）
        drift = abs(curr.hlc_wall_ms - prev.hlc_wall_ms)
        # 最大観測ドリフトを更新する
        if drift > max_observed:
            max_observed = drift
        # 差分が閾値を超えた場合は違反を記録する
        if drift > max_drift_ms:
            violation = DriftViolation(
                violation_type=ViolationType.WALL_CLOCK_DRIFT,
                axis_a=prev.axis_name,
                event_a=prev.event_id,
                axis_b=curr.axis_name,
                event_b=curr.event_id,
                hlc_a=_hlc_key(prev),
                hlc_b=_hlc_key(curr),
                delta_ms=drift,
                description=(
                    f"壁時計ドリフト {drift}ms が閾値 {max_drift_ms}ms を超えています: "
                    f"{prev.axis_name}/{prev.event_id} -> {curr.axis_name}/{curr.event_id}"
                ),
            )
            # 違反を violations リストに追加する
            violations.append(violation)
    # 最大観測ドリフトを返す
    return max_observed


def _group_events_by_node(
    events: Sequence[AxisEvent],
) -> Dict[Tuple[str, int], List[AxisEvent]]:
    """
    イベントを (axis_name, hlc_node_id) のペアごとにグループ化して返す。

    同一ノードのイベントは HLC キー順にソートされる。
    """
    # グループ辞書を初期化する
    groups: Dict[Tuple[str, int], List[AxisEvent]] = {}
    # 各イベントをグループに振り分ける
    for event in events:
        # ノードキーを計算する
        node_key = (event.axis_name, event.hlc_node_id)
        # グループが存在しない場合は新規作成する
        if node_key not in groups:
            groups[node_key] = []
        # イベントをグループに追加する
        groups[node_key].append(event)
    # 各グループを HLC キー順にソートする
    for key in groups:
        groups[key].sort(key=_hlc_key)
    # グループ辞書を返す
    return groups


def _count_axis_pairs(axes: Sequence[str]) -> int:
    """軸名リストから比較可能な軸ペアの数を計算して返す。"""
    # n 軸の組み合わせ数は n*(n-1)/2 （自己比較を除く）
    n = len(set(axes))
    # n が 2 未満の場合はペアが存在しない
    if n < 2:
        return 0
    # 組み合わせ数を返す
    return n * (n - 1) // 2


def verify_cross_axis_ordering(
    events: Sequence[AxisEvent],
    max_drift_ms: int = DEFAULT_MAX_DRIFT_MS,
    check_monotonicity: bool = True,
    check_drift: bool = True,
) -> HLCDriftResult:
    """
    複数軸のイベント列を検証して HLC ドリフト・因果順序違反を検出する。

    引数:
      events            : 収集した AxisEvent のリスト（HLC キー順に事前ソート推奨）
      max_drift_ms      : 壁時計ドリフトの許容上限（ミリ秒、デフォルト 5000ms）
      check_monotonicity: True の場合、同一ノード内の単調性違反を検出する
      check_drift       : True の場合、軸間の壁時計ドリフトを検出する

    戻り値:
      HLCDriftResult: 検証結果（違反リスト、最大ドリフト、統計情報）
    """
    # 違反を格納するリストを初期化する
    violations: List[DriftViolation] = []
    # 最大観測ドリフトを初期化する
    max_observed_drift = 0
    # events が空の場合は空の結果を返す
    if not events:
        logger.info("検証対象のイベントがありません")
        return HLCDriftResult(
            violations=[],
            max_observed_drift_ms=0,
            axis_pairs_checked=0,
            total_events_analyzed=0,
            axes_present=[],
        )
    # イベントリストを HLC キー順にソートする（入力が未ソートの場合に備える）
    sorted_events = sorted(events, key=_hlc_key)
    # 存在する軸名の集合を収集する
    axes_present = list({e.axis_name for e in sorted_events})
    # 軸ペアの数を計算する
    axis_pairs = _count_axis_pairs(axes_present)
    # 単調性チェックが有効な場合はノード別グループを構築する
    if check_monotonicity:
        # イベントをノード別にグループ化する
        events_by_node = _group_events_by_node(sorted_events)
        # 単調性違反を検出する
        _check_monotonicity(events_by_node, violations)
        # 単調性チェックの結果をデバッグログに記録する
        logger.debug(
            "単調性チェック完了: %d ノード, %d 違反",
            len(events_by_node),
            len([v for v in violations if v.violation_type == ViolationType.MONOTONICITY_VIOLATION]),
        )
    # 壁時計ドリフトチェックが有効な場合は隣接イベント間のドリフトを検出する
    if check_drift:
        # 壁時計ドリフト違反を検出する（最大観測ドリフトを更新する）
        observed = _check_wall_clock_drift(sorted_events, max_drift_ms, violations)
        # 最大観測ドリフトを更新する
        if observed > max_observed_drift:
            max_observed_drift = observed
        # ドリフトチェックの結果をデバッグログに記録する
        logger.debug(
            "壁時計ドリフトチェック完了: max_observed=%dms, 閾値=%dms, %d 違反",
            observed,
            max_drift_ms,
            len([v for v in violations if v.violation_type == ViolationType.WALL_CLOCK_DRIFT]),
        )
    # 因果順序違反チェック: HLC キー順にソートされたイベント列で同一軸内の逆転を検出する
    axis_last_event: Dict[str, AxisEvent] = {}
    # ソートされたイベント列を順に処理する
    for event in sorted_events:
        # この軸の直前イベントが存在する場合は因果順序を確認する
        if event.axis_name in axis_last_event:
            # 直前イベントを取得する
            prev = axis_last_event[event.axis_name]
            # HLC キーが前のイベント以上であることを確認する（単調増加）
            if _hlc_key(event) < _hlc_key(prev):
                # 因果順序違反を記録する
                delta = abs(prev.hlc_wall_ms - event.hlc_wall_ms)
                violation = DriftViolation(
                    violation_type=ViolationType.CAUSAL_VIOLATION,
                    axis_a=prev.axis_name,
                    event_a=prev.event_id,
                    axis_b=event.axis_name,
                    event_b=event.event_id,
                    hlc_a=_hlc_key(prev),
                    hlc_b=_hlc_key(event),
                    delta_ms=delta,
                    description=(
                        f"軸 '{event.axis_name}' で因果順序違反: "
                        f"{prev.event_id} ({_hlc_key(prev)}) の後に "
                        f"{event.event_id} ({_hlc_key(event)}) が来ているべきだが逆転"
                    ),
                )
                # 違反を violations リストに追加する
                violations.append(violation)
                # 最大観測ドリフトを更新する
                if delta > max_observed_drift:
                    max_observed_drift = delta
        # この軸の最後のイベントとして記録する
        axis_last_event[event.axis_name] = event
    # 検証結果を構築して返す
    result = HLCDriftResult(
        violations=violations,
        max_observed_drift_ms=max_observed_drift,
        axis_pairs_checked=axis_pairs,
        total_events_analyzed=len(sorted_events),
        axes_present=axes_present,
    )
    # 検証結果のサマリをデバッグログに記録する
    logger.debug("verify_cross_axis_ordering 完了:\n%s", result.summary())
    # 結果を返す
    return result


def verify_from_files(
    log_dir: str,
    axes: Optional[Sequence[str]] = None,
    max_drift_ms: int = DEFAULT_MAX_DRIFT_MS,
    time_range: Optional[Tuple[int, int]] = None,
) -> HLCDriftResult:
    """
    ファイルシステム上のイベントログを読み込んで検証を実行する便利関数。

    引数:
      log_dir    : イベントログファイルが格納されたディレクトリパス
      axes       : 検証対象の軸名リスト（None の場合は KNOWN_AXES を使用）
      max_drift_ms : 壁時計ドリフトの許容上限（ミリ秒）
      time_range : (start_wall_ms, end_wall_ms) のタプル（None の場合はフィルタなし）

    戻り値:
      HLCDriftResult: 検証結果
    """
    # pathlib.Path をインポートする
    from pathlib import Path
    # event_collector から EventCollector をインポートする
    from .event_collector import EventCollector
    # EventCollector を初期化する
    collector = EventCollector(log_dir=Path(log_dir))
    # イベントを収集する
    events = collector.collect(axes=axes, time_range=time_range)
    # 収集結果をデバッグログに記録する
    logger.debug(
        "ファイルから %d イベントを収集しました (ディレクトリ=%s)",
        len(events),
        log_dir,
    )
    # 検証を実行して結果を返す
    return verify_cross_axis_ordering(events, max_drift_ms=max_drift_ms)
