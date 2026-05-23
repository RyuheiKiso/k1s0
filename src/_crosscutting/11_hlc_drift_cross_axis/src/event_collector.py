#!/usr/bin/env python3
"""
event_collector.py — k1s0 cross-axis HLC イベントログ収集器

製造業プラットフォーム k1s0 の各軸（tier1/tier2/tier3/infra/data/security/ops/client）が
出力する JSONL 形式のイベントログを読み込み、HLC タイムスタンプ順にソートして返す。

収集したイベントは hlc_drift_verifier.py の verify_cross_axis_ordering() に渡して
軸間の因果順序違反（ドリフト）を検出するために使用する。

イベントログ形式（JSONL 1 行 = 1 イベント）:
  {"axis": "tier1", "event_id": "e001", "hlc_wall_ms": 1700000000000, "hlc_logical": 0,
   "hlc_node_id": 1, "payload_hash": "sha256:abcdef...", "ts_iso": "2026-05-23T00:00:00Z"}

各フィールドの説明:
  axis         : イベントを発行した軸の名前（tier1, tier2, ..., client）
  event_id     : 軸内でユニークなイベント識別子
  hlc_wall_ms  : HLC の wall clock 部分（UNIX epoch からの ms）
  hlc_logical  : HLC の logical カウンタ（0-65535）
  hlc_node_id  : HLC の node 識別子（0-65535）
  payload_hash : ペイロードの SHA-256 ハッシュ（改ざん検出用）
  ts_iso       : デバッグ用の ISO 8601 文字列（検証には使用しない）
"""

from __future__ import annotations

# dataclasses: AxisEvent の不変値オブジェクトを定義するために使用する
import dataclasses
# json: JSONL 形式のイベントログを解析するために使用する
import json
# logging: 収集過程のデバッグ情報を出力するために使用する
import logging
# pathlib: ファイルパスをプラットフォーム非依存に扱うために使用する
from pathlib import Path
# typing: 型ヒントに使用する
from typing import Dict, Iterator, List, Optional, Sequence, Tuple

# ロガーを初期化する（モジュール名をロガー名として使用する）
logger = logging.getLogger(__name__)

# 収集対象の軸名リストを定義する（k1s0 の 10 実装軸に対応する）
KNOWN_AXES: Tuple[str, ...] = (
    "tier1",
    "tier2",
    "tier3",
    "infra",
    "data",
    "security",
    "ops",
    "client",
    "test",
    "formal",
)

# デフォルトのイベントログファイル名パターンを定義する（軸名 + suffix）
DEFAULT_LOG_SUFFIX: str = "_events.jsonl"

# イベントログの読み取り最大行数（メモリ保護のために上限を設ける）
MAX_LINES_PER_FILE: int = 1_000_000


@dataclasses.dataclass(frozen=True, order=False)
class AxisEvent:
    """
    単一軸から収集した HLC タイムスタンプ付きイベントを表す不変データクラス。

    HLC タイムスタンプは (hlc_wall_ms, hlc_logical, hlc_node_id) の 3 tuple で表現する。
    比較演算子は HLC の全順序（wall_ms → logical → node_id の辞書順）に従う。
    """

    # axis_name: イベントを発行した軸の名前（tier1, tier2, ..., client など）
    axis_name: str
    # event_id: 軸内でユニークなイベント識別子（例: "e001", "req-abc-123"）
    event_id: str
    # hlc_wall_ms: HLC の wall clock 部分（UNIX epoch からの ms）
    hlc_wall_ms: int
    # hlc_logical: HLC の logical カウンタ（0 以上 65535 以下）
    hlc_logical: int
    # hlc_node_id: HLC の node 識別子（0 以上 65535 以下）
    hlc_node_id: int
    # payload_hash: ペイロードの SHA-256 ハッシュ（改ざん検出用、例: "sha256:abcdef..."）
    payload_hash: str

    def hlc_key(self) -> Tuple[int, int, int]:
        """HLC 全順序比較のためのキータプルを返す。"""
        # (hlc_wall_ms, hlc_logical, hlc_node_id) の辞書順でキーを構築する
        return (self.hlc_wall_ms, self.hlc_logical, self.hlc_node_id)

    def __lt__(self, other: object) -> bool:
        # other が AxisEvent でない場合は NotImplemented を返す
        if not isinstance(other, AxisEvent):
            return NotImplemented
        # HLC キーで辞書順比較を行う
        return self.hlc_key() < other.hlc_key()

    def __le__(self, other: object) -> bool:
        # other が AxisEvent でない場合は NotImplemented を返す
        if not isinstance(other, AxisEvent):
            return NotImplemented
        # HLC キーで辞書順比較（以下）を行う
        return self.hlc_key() <= other.hlc_key()

    def __gt__(self, other: object) -> bool:
        # other が AxisEvent でない場合は NotImplemented を返す
        if not isinstance(other, AxisEvent):
            return NotImplemented
        # HLC キーで辞書順比較（より大きい）を行う
        return self.hlc_key() > other.hlc_key()

    def __ge__(self, other: object) -> bool:
        # other が AxisEvent でない場合は NotImplemented を返す
        if not isinstance(other, AxisEvent):
            return NotImplemented
        # HLC キーで辞書順比較（以上）を行う
        return self.hlc_key() >= other.hlc_key()

    def __eq__(self, other: object) -> bool:
        # other が AxisEvent でない場合は NotImplemented を返す
        if not isinstance(other, AxisEvent):
            return NotImplemented
        # 全フィールドが同一の場合のみ等値とする
        return (
            self.axis_name == other.axis_name
            and self.event_id == other.event_id
            and self.hlc_key() == other.hlc_key()
            and self.payload_hash == other.payload_hash
        )

    def __hash__(self) -> int:
        # frozen dataclass として hash を定義する
        return hash((self.axis_name, self.event_id, self.hlc_key(), self.payload_hash))


def _parse_jsonl_line(line: str, source_axis: str, line_num: int) -> Optional[AxisEvent]:
    """
    JSONL の 1 行を解析して AxisEvent を返す。

    パースに失敗した場合は None を返す（ログは warning レベルで出力する）。
    """
    # 空行やコメント行をスキップする
    stripped = line.strip()
    # 空行の場合は None を返す（エラーではない）
    if not stripped:
        return None
    # '#' で始まる行はコメントとしてスキップする
    if stripped.startswith("#"):
        return None
    # JSON のデコードを試みる
    try:
        obj = json.loads(stripped)
    except json.JSONDecodeError as exc:
        # JSON デコードエラーを warning ログに記録する
        logger.warning(
            "JSONL デコードエラー: %s (軸=%s, 行=%d, エラー=%s)",
            stripped[:80],
            source_axis,
            line_num,
            exc,
        )
        return None
    # obj が辞書でない場合は警告を出して None を返す
    if not isinstance(obj, dict):
        logger.warning(
            "JSONL 行がオブジェクトではありません: %s (軸=%s, 行=%d)",
            stripped[:80],
            source_axis,
            line_num,
        )
        return None
    # 必須フィールドを取得する
    try:
        # axis_name はファイルの source_axis を優先し、フィールドが存在すれば上書きする
        axis_name = str(obj.get("axis", source_axis))
        # event_id は文字列として取得する
        event_id = str(obj["event_id"])
        # hlc_wall_ms は整数として取得する
        hlc_wall_ms = int(obj["hlc_wall_ms"])
        # hlc_logical は整数として取得する（省略時は 0 を使用する）
        hlc_logical = int(obj.get("hlc_logical", 0))
        # hlc_node_id は整数として取得する（省略時は 0 を使用する）
        hlc_node_id = int(obj.get("hlc_node_id", 0))
        # payload_hash は文字列として取得する（省略時は空文字列を使用する）
        payload_hash = str(obj.get("payload_hash", ""))
    except (KeyError, TypeError, ValueError) as exc:
        # 必須フィールドが欠けている場合は warning ログに記録する
        logger.warning(
            "AxisEvent フィールド欠落: (軸=%s, 行=%d, エラー=%s)",
            source_axis,
            line_num,
            exc,
        )
        return None
    # AxisEvent を生成して返す
    try:
        return AxisEvent(
            axis_name=axis_name,
            event_id=event_id,
            hlc_wall_ms=hlc_wall_ms,
            hlc_logical=hlc_logical,
            hlc_node_id=hlc_node_id,
            payload_hash=payload_hash,
        )
    except Exception as exc:
        # AxisEvent 生成に失敗した場合は warning ログに記録する
        logger.warning(
            "AxisEvent 生成失敗: (軸=%s, 行=%d, エラー=%s)",
            source_axis,
            line_num,
            exc,
        )
        return None


def _read_jsonl_file(path: Path, axis_name: str) -> Iterator[AxisEvent]:
    """
    指定したファイルから AxisEvent を逐次読み込むジェネレータ。

    MAX_LINES_PER_FILE を超えた場合は warning を出して打ち切る。
    """
    # ファイルの存在を確認する
    if not path.exists():
        logger.warning("イベントログファイルが存在しません: %s (軸=%s)", path, axis_name)
        return
    # ファイルを UTF-8 で開く（エンコードエラーは replace で処理する）
    try:
        fh = path.open(encoding="utf-8", errors="replace")
    except OSError as exc:
        # ファイルオープンに失敗した場合は warning ログに記録する
        logger.warning(
            "イベントログファイルをオープンできません: %s (エラー=%s)",
            path,
            exc,
        )
        return
    # ファイルハンドルを使って 1 行ずつ読み込む
    with fh:
        # 行番号カウンタを初期化する
        line_num = 0
        # 1 行ずつ処理する
        for line in fh:
            # 行番号を増加させる
            line_num += 1
            # 最大行数を超えた場合は打ち切る
            if line_num > MAX_LINES_PER_FILE:
                logger.warning(
                    "最大行数 (%d) を超えたため読み込みを打ち切ります: %s (軸=%s)",
                    MAX_LINES_PER_FILE,
                    path,
                    axis_name,
                )
                break
            # 1 行をパースして AxisEvent を生成する
            event = _parse_jsonl_line(line, axis_name, line_num)
            # パースに成功した場合のみ yield する
            if event is not None:
                yield event


class EventCollector:
    """
    複数軸のイベントログを収集して HLC タイムスタンプ順に返すコレクタ。

    使用例:
      collector = EventCollector(log_dir=Path("./logs"))
      events = collector.collect(axes=["tier1", "tier2"], time_range=(start_ms, end_ms))
    """

    def __init__(
        self,
        log_dir: Path,
        log_suffix: str = DEFAULT_LOG_SUFFIX,
    ) -> None:
        """ログディレクトリとファイル suffix を受け取って EventCollector を初期化する。"""
        # log_dir をインスタンス変数に保存する
        self._log_dir: Path = log_dir
        # log_suffix をインスタンス変数に保存する
        self._log_suffix: str = log_suffix
        # 収集した軸ごとのイベント数を記録する辞書を初期化する
        self._axis_counts: Dict[str, int] = {}

    def _log_path_for_axis(self, axis_name: str) -> Path:
        """指定した軸のイベントログファイルパスを返す。"""
        # ログディレクトリ + 軸名 + suffix でパスを構築する
        return self._log_dir / f"{axis_name}{self._log_suffix}"

    def collect(
        self,
        axes: Optional[Sequence[str]] = None,
        time_range: Optional[Tuple[int, int]] = None,
    ) -> List[AxisEvent]:
        """
        指定した軸のイベントを収集して HLC タイムスタンプ順にソートしたリストを返す。

        引数:
          axes       : 収集する軸名のリスト（None の場合は KNOWN_AXES を全て収集する）
          time_range : (start_wall_ms, end_wall_ms) のタプル（両端を含む）。
                       None の場合はフィルタなしで全イベントを収集する

        戻り値:
          HLC タイムスタンプ（wall_ms → logical → node_id）でソートされた AxisEvent のリスト
        """
        # 収集対象の軸リストを決定する
        target_axes: Sequence[str] = axes if axes is not None else KNOWN_AXES
        # 収集したすべてのイベントを格納するリストを初期化する
        all_events: List[AxisEvent] = []
        # 各軸のイベントログを読み込む
        for axis_name in target_axes:
            # 対象ファイルのパスを計算する
            log_path = self._log_path_for_axis(axis_name)
            # 軸ごとのイベント数をカウントするための変数を初期化する
            axis_count = 0
            # ファイルから AxisEvent を逐次読み込む
            for event in _read_jsonl_file(log_path, axis_name):
                # time_range が指定されている場合はフィルタリングする
                if time_range is not None:
                    # start_wall_ms より前のイベントはスキップする
                    if event.hlc_wall_ms < time_range[0]:
                        continue
                    # end_wall_ms より後のイベントはスキップする
                    if event.hlc_wall_ms > time_range[1]:
                        continue
                # フィルタを通過したイベントを追加する
                all_events.append(event)
                # 軸ごとのカウントを増加させる
                axis_count += 1
            # 収集した軸ごとのイベント数を記録する
            self._axis_counts[axis_name] = axis_count
            # 収集結果をデバッグログに記録する
            logger.debug(
                "軸 '%s' から %d イベントを収集しました",
                axis_name,
                axis_count,
            )
        # 全イベントを HLC タイムスタンプ順にソートする（hlc_key() の辞書順）
        all_events.sort(key=lambda e: e.hlc_key())
        # ソートしたイベントリストを返す
        return all_events

    def collect_from_files(
        self,
        file_map: Dict[str, Path],
        time_range: Optional[Tuple[int, int]] = None,
    ) -> List[AxisEvent]:
        """
        軸名とファイルパスのマッピングを受け取り、イベントを収集して返す。

        collect() よりも柔軟なインターフェースで、任意のファイルパスを指定できる。

        引数:
          file_map   : {軸名: ファイルパス} のマッピング
          time_range : (start_wall_ms, end_wall_ms) のタプル（None の場合はフィルタなし）

        戻り値:
          HLC タイムスタンプ順にソートされた AxisEvent のリスト
        """
        # 収集したすべてのイベントを格納するリストを初期化する
        all_events: List[AxisEvent] = []
        # 各軸とファイルパスのペアを処理する
        for axis_name, log_path in file_map.items():
            # 軸ごとのイベント数をカウントするための変数を初期化する
            axis_count = 0
            # ファイルから AxisEvent を逐次読み込む
            for event in _read_jsonl_file(log_path, axis_name):
                # time_range が指定されている場合はフィルタリングする
                if time_range is not None:
                    # 範囲外のイベントはスキップする
                    if not (time_range[0] <= event.hlc_wall_ms <= time_range[1]):
                        continue
                # フィルタを通過したイベントを追加する
                all_events.append(event)
                # 軸ごとのカウントを増加させる
                axis_count += 1
            # 収集した軸ごとのイベント数を記録する
            self._axis_counts[axis_name] = axis_count
        # 全イベントを HLC タイムスタンプ順にソートする
        all_events.sort(key=lambda e: e.hlc_key())
        # ソートしたイベントリストを返す
        return all_events

    @property
    def axis_counts(self) -> Dict[str, int]:
        """最後の collect/collect_from_files 呼び出し時の軸ごとのイベント数を返す。"""
        # コピーを返して内部状態の直接変更を防ぐ
        return dict(self._axis_counts)

    def total_events(self) -> int:
        """最後の collect/collect_from_files 呼び出し時の合計イベント数を返す。"""
        # 全軸のイベント数を合計して返す
        return sum(self._axis_counts.values())
