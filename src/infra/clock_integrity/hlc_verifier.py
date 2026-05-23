"""src/infra/clock_integrity/hlc_verifier.py

クラスター全体の時刻整合性を検証する HLC ドリフト検証器。
仕様: 13_時刻整合適合仕様.md §clock_integrity_class

- ClockReading: 各ノードの壁時計・HLC wall_ms・HLC logical カウンタを保持する
- ClockSkewResult: 最大スキュー・違反ペア・閾値内判定を保持する
- HLCVerifier: 単調性チェック・ペアワイズスキュー検証・異常値検出・レポート生成を提供する
- chrony NTP データと連携してシステム時刻の誤差を定量評価する
"""

from __future__ import annotations

import logging
import math
import re
import subprocess
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

# モジュールロガーを初期化する
logger = logging.getLogger(__name__)

# デフォルトの最大許容スキュー閾値（マイクロ秒）: 500 us
_DEFAULT_SKEW_LIMIT_US = 500
# 突発ジャンプを異常と判定する閾値（マイクロ秒）: 1 秒 = 1,000,000 us
_JUMP_ANOMALY_THRESHOLD_US = 1_000_000
# HLC の最大許容論理カウンタ値（これを超えると壁時計停滞とみなす）
_MAX_LOGICAL_COUNTER = 10_000


# ---------------------------------------------------------------------------
# ClockReading データクラス
# ---------------------------------------------------------------------------

@dataclass
class ClockReading:
    """クラスター内の 1 ノードの時刻読み取り値を保持するデータクラス。"""

    # ノードの識別子（hostname または k8s node name）
    node_id: str
    # ノードの壁時計ナノ秒タイムスタンプ（UNIX epoch からの経過ナノ秒）
    wall_clock_ns: int
    # HLC の壁時計部（ミリ秒精度）
    hlc_wall_ms: int
    # HLC の論理カウンタ（同一壁時計内の順序付けに使用する）
    hlc_logical: int
    # 読み取り元のソース識別子（"chrony" / "ptp" / "direct" 等）
    source: str
    # chrony の推定誤差（ナノ秒、取得できない場合は -1）
    chrony_rms_offset_ns: int = -1
    # NTP 層番号（stratum）
    ntp_stratum: int = -1
    # 読み取り時刻（Unix タイムスタンプ、メタデータとして記録する）
    collected_at: float = field(default_factory=time.time)

    @property
    def wall_clock_us(self) -> int:
        """壁時計をマイクロ秒単位で返す（ns → us 変換）。"""
        # ナノ秒を 1000 で割ってマイクロ秒に変換する
        return self.wall_clock_ns // 1_000

    @property
    def wall_clock_ms(self) -> int:
        """壁時計をミリ秒単位で返す（ns → ms 変換）。"""
        # ナノ秒を 1,000,000 で割ってミリ秒に変換する
        return self.wall_clock_ns // 1_000_000

    def to_dict(self) -> dict[str, Any]:
        """ClockReading を辞書形式にシリアライズする。"""
        return {
            "node_id": self.node_id,
            "wall_clock_ns": self.wall_clock_ns,
            "hlc_wall_ms": self.hlc_wall_ms,
            "hlc_logical": self.hlc_logical,
            "source": self.source,
            "chrony_rms_offset_ns": self.chrony_rms_offset_ns,
            "ntp_stratum": self.ntp_stratum,
            "collected_at": self.collected_at,
        }


# ---------------------------------------------------------------------------
# SkewPair データクラス
# ---------------------------------------------------------------------------

@dataclass
class SkewPair:
    """2 ノード間のスキューを表すデータクラス。"""

    # ノード A の識別子
    node_a: str
    # ノード B の識別子
    node_b: str
    # 2 ノード間のスキュー（マイクロ秒、絶対値）
    skew_us: int
    # スキューが閾値を超過しているか
    exceeds_limit: bool

    def to_dict(self) -> dict[str, Any]:
        """SkewPair を辞書形式にシリアライズする。"""
        return {
            "node_a": self.node_a,
            "node_b": self.node_b,
            "skew_us": self.skew_us,
            "exceeds_limit": self.exceeds_limit,
        }


# ---------------------------------------------------------------------------
# ClockSkewResult データクラス
# ---------------------------------------------------------------------------

@dataclass
class ClockSkewResult:
    """クラスター全体のスキュー検証結果を保持するデータクラス。"""

    # クラスター内の最大スキュー（マイクロ秒）
    max_skew_us: int
    # 閾値を超過したノードペアのリスト
    violating_pairs: list[SkewPair]
    # 全ペアが制限内に収まっているか
    within_limit: bool
    # 使用した閾値（マイクロ秒）
    limit_us: int

    def to_dict(self) -> dict[str, Any]:
        """ClockSkewResult を辞書形式にシリアライズする。"""
        return {
            "max_skew_us": self.max_skew_us,
            "within_limit": self.within_limit,
            "limit_us": self.limit_us,
            "violating_pair_count": len(self.violating_pairs),
            "violating_pairs": [p.to_dict() for p in self.violating_pairs],
        }


# ---------------------------------------------------------------------------
# MonotonicityViolation データクラス
# ---------------------------------------------------------------------------

@dataclass
class MonotonicityViolation:
    """HLC 単調性違反を表すデータクラス。"""

    # 違反が発生したノード ID
    node_id: str
    # 前の読み取り値（時間的に先行する）
    prev_hlc_wall_ms: int
    # 現在の読み取り値（時間的に後行するべき）
    curr_hlc_wall_ms: int
    # 後退量（ミリ秒）
    regression_ms: int

    def to_dict(self) -> dict[str, Any]:
        """MonotonicityViolation を辞書形式にシリアライズする。"""
        return {
            "node_id": self.node_id,
            "prev_hlc_wall_ms": self.prev_hlc_wall_ms,
            "curr_hlc_wall_ms": self.curr_hlc_wall_ms,
            "regression_ms": self.regression_ms,
        }


# ---------------------------------------------------------------------------
# AnomalyDetection データクラス
# ---------------------------------------------------------------------------

@dataclass
class ClockAnomaly:
    """検出されたクロック異常を表すデータクラス。"""

    # 異常の種類（"jump_forward" / "jump_backward" / "high_logical" / "stale"）
    anomaly_type: str
    # 異常が発生したノード ID
    node_id: str
    # 異常の規模（マイクロ秒または論理カウンタ値）
    magnitude: int
    # 異常の説明メッセージ
    description: str

    def to_dict(self) -> dict[str, Any]:
        """ClockAnomaly を辞書形式にシリアライズする。"""
        return {
            "anomaly_type": self.anomaly_type,
            "node_id": self.node_id,
            "magnitude": self.magnitude,
            "description": self.description,
        }


# ---------------------------------------------------------------------------
# VerifierReport データクラス
# ---------------------------------------------------------------------------

@dataclass
class VerifierReport:
    """HLCVerifier の検証結果をまとめるレポートデータクラス。"""

    # 検証実行時刻（Unix タイムスタンプ）
    verified_at: float
    # 検証対象のノード数
    node_count: int
    # 読み取り値の総数
    reading_count: int
    # スキュー検証結果
    skew_result: ClockSkewResult | None
    # 単調性違反のリスト
    monotonicity_violations: list[MonotonicityViolation]
    # 検出された異常のリスト
    anomalies: list[ClockAnomaly]
    # 全体の健全性判定（全チェックが通過した場合 True）
    healthy: bool

    def to_dict(self) -> dict[str, Any]:
        """VerifierReport を辞書形式にシリアライズする。"""
        # 全フィールドを辞書にマッピングする
        return {
            "verified_at": self.verified_at,
            "node_count": self.node_count,
            "reading_count": self.reading_count,
            "skew_result": self.skew_result.to_dict() if self.skew_result else None,
            "monotonicity_violation_count": len(self.monotonicity_violations),
            "monotonicity_violations": [v.to_dict() for v in self.monotonicity_violations],
            "anomaly_count": len(self.anomalies),
            "anomalies": [a.to_dict() for a in self.anomalies],
            "healthy": self.healthy,
        }


# ---------------------------------------------------------------------------
# ChronyData データクラス
# ---------------------------------------------------------------------------

@dataclass
class ChronyData:
    """chrony から取得した NTP 追跡データを保持するデータクラス。"""

    # 参照 NTP サーバーのアドレス
    reference_id: str
    # 推定システム時刻誤差（秒）
    rms_offset_s: float
    # 現在のスキュー推定（ppm）
    freq_error_ppm: float
    # NTP 階層（stratum）
    stratum: int
    # leap status（"Normal" / "Add second" / "Delete second" / "Not synchronised"）
    leap_status: str

    def rms_offset_us(self) -> int:
        """RMS オフセットをマイクロ秒単位で返す。"""
        # 秒からマイクロ秒に変換する（絶対値を取る）
        return int(abs(self.rms_offset_s) * 1_000_000)

    def to_dict(self) -> dict[str, Any]:
        """ChronyData を辞書形式にシリアライズする。"""
        return {
            "reference_id": self.reference_id,
            "rms_offset_s": self.rms_offset_s,
            "freq_error_ppm": self.freq_error_ppm,
            "stratum": self.stratum,
            "leap_status": self.leap_status,
            "rms_offset_us": self.rms_offset_us(),
        }


# ---------------------------------------------------------------------------
# HLCVerifier クラス
# ---------------------------------------------------------------------------

class HLCVerifier:
    """クラスター全体の HLC 整合性を検証するクラス。

    各ノードから収集した ClockReading を蓄積し、単調性・ペアワイズスキュー・
    突発ジャンプの 3 軸で時刻整合性を検証する。
    """

    def __init__(self, skew_limit_us: int = _DEFAULT_SKEW_LIMIT_US, dry_run: bool = True) -> None:
        """HLCVerifier を初期化する。

        Args:
            skew_limit_us: 許容する最大スキュー（マイクロ秒）
            dry_run: True の場合は chronyc 等の外部コマンドを実行しない
        """
        # 許容スキュー閾値を保持する
        self._skew_limit_us = skew_limit_us
        # dry-run モードフラグを保持する
        self._dry_run = dry_run
        # ノード ID → 最新 ClockReading のマッピングを初期化する
        self._readings: dict[str, ClockReading] = {}
        # ノード ID → 前回の ClockReading（単調性チェック用）のマッピングを初期化する
        self._prev_readings: dict[str, ClockReading] = {}
        # 累積した全 ClockReading のリストを初期化する
        self._all_readings: list[ClockReading] = []

    def add_reading(self, reading: ClockReading) -> None:
        """ノードの時刻読み取り値を検証器に追加する。

        Args:
            reading: 追加する ClockReading
        """
        # 前回の読み取り値を保存する（単調性チェック用）
        if reading.node_id in self._readings:
            self._prev_readings[reading.node_id] = self._readings[reading.node_id]
        # 最新の読み取り値を更新する
        self._readings[reading.node_id] = reading
        # 全履歴に追加する
        self._all_readings.append(reading)
        logger.debug(
            "HLCVerifier: 読み取り追加 node=%s hlc=%d:%d wall_ns=%d",
            reading.node_id, reading.hlc_wall_ms, reading.hlc_logical, reading.wall_clock_ns
        )

    def check_monotonicity(self) -> list[MonotonicityViolation]:
        """各ノードの HLC 単調性を検証して違反リストを返す。

        HLC は時間が経過するにつれて単調増加しなければならない。
        hlc_wall_ms が後退している場合を違反として検出する。

        Returns:
            検出された単調性違反のリスト（正常な場合は空リスト）
        """
        # 単調性違反を格納するリストを初期化する
        violations: list[MonotonicityViolation] = []
        # 前回読み取りが存在するノードについて確認する
        for node_id, curr in self._readings.items():
            if node_id not in self._prev_readings:
                continue
            prev = self._prev_readings[node_id]
            # 現在の hlc_wall_ms が前回より小さい場合は単調性違反
            if curr.hlc_wall_ms < prev.hlc_wall_ms:
                regression_ms = prev.hlc_wall_ms - curr.hlc_wall_ms
                violation = MonotonicityViolation(
                    node_id=node_id,
                    prev_hlc_wall_ms=prev.hlc_wall_ms,
                    curr_hlc_wall_ms=curr.hlc_wall_ms,
                    regression_ms=regression_ms,
                )
                violations.append(violation)
                logger.warning(
                    "HLCVerifier: 単調性違反 node=%s 後退=%d ms",
                    node_id, regression_ms
                )
        return violations

    def check_pairwise_skew(self, limit_us: int | None = None) -> ClockSkewResult:
        """全ノードペアのスキューを計算して閾値超過を検出する。

        全ノードペアの壁時計差（マイクロ秒）を計算し、最大スキューと
        閾値超過ペアを特定する。

        Args:
            limit_us: 使用する閾値（None の場合はコンストラクタの設定値を使用）

        Returns:
            ClockSkewResult
        """
        # 閾値を決定する
        threshold = limit_us if limit_us is not None else self._skew_limit_us
        # 現在の読み取り値リストを取得する
        node_readings = list(self._readings.values())
        # ノード数が 2 未満の場合はチェック不可
        if len(node_readings) < 2:
            return ClockSkewResult(
                max_skew_us=0,
                violating_pairs=[],
                within_limit=True,
                limit_us=threshold,
            )
        # 全ペアのスキューを計算する
        max_skew_us = 0
        violating_pairs: list[SkewPair] = []
        # i < j の全組み合わせを評価する
        for i in range(len(node_readings)):
            for j in range(i + 1, len(node_readings)):
                reading_a = node_readings[i]
                reading_b = node_readings[j]
                # 壁時計差を絶対値マイクロ秒で計算する
                skew_ns = abs(reading_a.wall_clock_ns - reading_b.wall_clock_ns)
                skew_us = skew_ns // 1_000
                # 最大スキューを更新する
                if skew_us > max_skew_us:
                    max_skew_us = skew_us
                # 閾値超過の場合は違反ペアリストに追加する
                if skew_us > threshold:
                    pair = SkewPair(
                        node_a=reading_a.node_id,
                        node_b=reading_b.node_id,
                        skew_us=skew_us,
                        exceeds_limit=True,
                    )
                    violating_pairs.append(pair)
                    logger.warning(
                        "HLCVerifier: スキュー超過 nodes=%s↔%s skew=%d us limit=%d us",
                        reading_a.node_id, reading_b.node_id, skew_us, threshold
                    )
        # 違反ペアが 0 件の場合は within_limit=True
        within_limit = len(violating_pairs) == 0
        return ClockSkewResult(
            max_skew_us=max_skew_us,
            violating_pairs=violating_pairs,
            within_limit=within_limit,
            limit_us=threshold,
        )

    def detect_anomalies(self) -> list[ClockAnomaly]:
        """突発的なクロックジャンプや異常な論理カウンタを検出する。

        以下の異常を検出する:
        - 前回読み取りから 1 秒以上の前方ジャンプ
        - 前回読み取りから 1 秒以上の後方ジャンプ
        - hlc_logical が _MAX_LOGICAL_COUNTER を超えた場合（壁時計停滞）
        - chrony 推定誤差が閾値を超えた場合

        Returns:
            検出された ClockAnomaly のリスト
        """
        # 異常リストを初期化する
        anomalies: list[ClockAnomaly] = []
        # 全ノードについてチェックする
        for node_id, curr in self._readings.items():
            # 前回読み取りとの比較が可能な場合のみジャンプチェックを実行する
            if node_id in self._prev_readings:
                prev = self._prev_readings[node_id]
                # 壁時計差をマイクロ秒で計算する
                delta_us = (curr.wall_clock_ns - prev.wall_clock_ns) // 1_000
                # 前方ジャンプ検出: 1 秒以上の急激な増加
                if delta_us > _JUMP_ANOMALY_THRESHOLD_US:
                    anomaly = ClockAnomaly(
                        anomaly_type="jump_forward",
                        node_id=node_id,
                        magnitude=delta_us,
                        description=f"前方ジャンプ {delta_us} us 検出 (閾値: {_JUMP_ANOMALY_THRESHOLD_US} us)",
                    )
                    anomalies.append(anomaly)
                    logger.error("HLCVerifier: 前方ジャンプ node=%s magnitude=%d us", node_id, delta_us)
                # 後方ジャンプ検出: 負方向の 1 秒以上の変化
                elif delta_us < -_JUMP_ANOMALY_THRESHOLD_US:
                    anomaly = ClockAnomaly(
                        anomaly_type="jump_backward",
                        node_id=node_id,
                        magnitude=abs(delta_us),
                        description=f"後方ジャンプ {abs(delta_us)} us 検出 (閾値: {_JUMP_ANOMALY_THRESHOLD_US} us)",
                    )
                    anomalies.append(anomaly)
                    logger.error("HLCVerifier: 後方ジャンプ node=%s magnitude=%d us", node_id, abs(delta_us))
            # 論理カウンタ異常検出: 壁時計が停滞している可能性を示す
            if curr.hlc_logical > _MAX_LOGICAL_COUNTER:
                anomaly = ClockAnomaly(
                    anomaly_type="high_logical",
                    node_id=node_id,
                    magnitude=curr.hlc_logical,
                    description=f"HLC logical カウンタ過大: {curr.hlc_logical} (閾値: {_MAX_LOGICAL_COUNTER})",
                )
                anomalies.append(anomaly)
                logger.warning("HLCVerifier: 論理カウンタ過大 node=%s logical=%d", node_id, curr.hlc_logical)
            # chrony 誤差異常検出: _DEFAULT_SKEW_LIMIT_US の 10 倍を超えた場合
            if curr.chrony_rms_offset_ns >= 0:
                offset_us = curr.chrony_rms_offset_ns // 1_000
                chrony_limit_us = _DEFAULT_SKEW_LIMIT_US * 10
                if offset_us > chrony_limit_us:
                    anomaly = ClockAnomaly(
                        anomaly_type="chrony_offset_high",
                        node_id=node_id,
                        magnitude=offset_us,
                        description=f"chrony RMS オフセット過大: {offset_us} us (閾値: {chrony_limit_us} us)",
                    )
                    anomalies.append(anomaly)
                    logger.warning("HLCVerifier: chrony オフセット過大 node=%s offset=%d us", node_id, offset_us)
        return anomalies

    def read_chrony_data(self, node_id: str = "local") -> ChronyData | None:
        """chronyc コマンドを実行して NTP 追跡データを取得する。

        dry_run モードの場合はダミーデータを返す。
        chronyc が利用できない場合は None を返す。

        Args:
            node_id: 読み取るノードの識別子（ログ用）

        Returns:
            取得した ChronyData、取得失敗時は None
        """
        # dry_run モードの場合はダミーデータを返す
        if self._dry_run:
            logger.debug("HLCVerifier: dry_run モード - chrony ダミーデータを返す node=%s", node_id)
            return ChronyData(
                reference_id="127.127.1.0",
                rms_offset_s=0.000050,
                freq_error_ppm=0.1,
                stratum=3,
                leap_status="Normal",
            )
        # chronyc tracking コマンドを実行する
        try:
            result = subprocess.run(
                ["chronyc", "tracking"],
                capture_output=True, text=True, timeout=5
            )
            # コマンドが失敗した場合は None を返す
            if result.returncode != 0:
                logger.error("HLCVerifier: chronyc 実行失敗 node=%s stderr=%s", node_id, result.stderr)
                return None
            # 出力をパースして ChronyData を構築する
            return self._parse_chrony_output(result.stdout)
        except FileNotFoundError:
            logger.warning("HLCVerifier: chronyc コマンドが見つからない node=%s", node_id)
            return None
        except Exception as exc:
            logger.error("HLCVerifier: chronyc 実行エラー node=%s error=%s", node_id, exc)
            return None

    def _parse_chrony_output(self, output: str) -> ChronyData | None:
        """chronyc tracking コマンドの出力をパースして ChronyData を返す。"""
        # パース結果を格納する辞書を初期化する
        data: dict[str, str] = {}
        for line in output.splitlines():
            if ":" in line:
                key, _, value = line.partition(":")
                data[key.strip()] = value.strip()
        # Reference ID を取得する
        reference_id = data.get("Reference ID", "unknown").split()[0]
        # RMS offset を取得してフロートに変換する
        rms_str = data.get("RMS offset", "0 seconds").split()[0]
        try:
            rms_offset_s = float(rms_str)
        except ValueError:
            rms_offset_s = 0.0
        # Frequency error を取得する
        freq_str = data.get("Frequency", "0 ppm").split()[0]
        try:
            freq_error_ppm = float(freq_str)
        except ValueError:
            freq_error_ppm = 0.0
        # Stratum を取得する
        stratum_str = data.get("Stratum", "0")
        try:
            stratum = int(stratum_str)
        except ValueError:
            stratum = 0
        # Leap status を取得する
        leap_status = data.get("Leap status", "Unknown")
        return ChronyData(
            reference_id=reference_id,
            rms_offset_s=rms_offset_s,
            freq_error_ppm=freq_error_ppm,
            stratum=stratum,
            leap_status=leap_status,
        )

    def generate_report(self) -> VerifierReport:
        """全検証チェックを実行して包括的なレポートを生成する。

        Returns:
            全チェック結果をまとめた VerifierReport
        """
        # 検証実行時刻を記録する
        verified_at = time.time()
        # 単調性チェックを実行する
        monotonicity_violations = self.check_monotonicity()
        # ペアワイズスキューチェックを実行する
        skew_result = self.check_pairwise_skew()
        # 異常値検出を実行する
        anomalies = self.detect_anomalies()
        # 全体の健全性を判定する（全チェックが通過した場合 True）
        healthy = (
            len(monotonicity_violations) == 0
            and skew_result.within_limit
            and len(anomalies) == 0
        )
        # VerifierReport を構築して返す
        report = VerifierReport(
            verified_at=verified_at,
            node_count=len(self._readings),
            reading_count=len(self._all_readings),
            skew_result=skew_result,
            monotonicity_violations=monotonicity_violations,
            anomalies=anomalies,
            healthy=healthy,
        )
        # レポート生成結果をログに記録する
        logger.info(
            "HLCVerifier: レポート生成完了 nodes=%d readings=%d healthy=%s violations=%d anomalies=%d",
            report.node_count, report.reading_count, report.healthy,
            len(monotonicity_violations), len(anomalies)
        )
        return report

    def get_node_count(self) -> int:
        """現在登録されているノード数を返す。"""
        return len(self._readings)

    def get_reading_count(self) -> int:
        """累積した読み取り値の総件数を返す。"""
        return len(self._all_readings)

    def clear(self) -> None:
        """全読み取り値をクリアしてリセットする。"""
        # 全読み取り辞書を空にする
        self._readings.clear()
        self._prev_readings.clear()
        self._all_readings.clear()
        logger.debug("HLCVerifier: 読み取り値をクリアした")

    def get_max_skew_us(self) -> int:
        """現在の読み取り値から最大スキューをマイクロ秒で返す。"""
        # ペアワイズスキューチェックを実行して最大値を返す
        result = self.check_pairwise_skew()
        return result.max_skew_us

    def is_cluster_healthy(self, limit_us: int | None = None) -> bool:
        """クラスター全体の時刻整合性が健全かどうかを判定する。"""
        # 読み取り値が存在しない場合は健全とみなす
        if not self._readings:
            return True
        # スキューチェックを実行する
        skew_result = self.check_pairwise_skew(limit_us)
        # 単調性チェックを実行する
        violations = self.check_monotonicity()
        # 全て通過した場合のみ健全と判定する
        return skew_result.within_limit and len(violations) == 0


# ---------------------------------------------------------------------------
# ClockReadingCollector クラス
# ---------------------------------------------------------------------------

class ClockReadingCollector:
    """複数ノードから ClockReading を収集して HLCVerifier に投入するコレクター。"""

    def __init__(self, verifier: HLCVerifier, dry_run: bool = True) -> None:
        """ClockReadingCollector を初期化する。

        Args:
            verifier: 読み取り値を投入する HLCVerifier インスタンス
            dry_run: True の場合は実際のコマンドを実行しない
        """
        # HLCVerifier インスタンスを保持する
        self._verifier = verifier
        # dry-run モードフラグを保持する
        self._dry_run = dry_run
        # 収集済みノード数カウンタを初期化する
        self._collected_nodes: int = 0

    def collect_local(self, node_id: str = "local") -> ClockReading:
        """ローカルノードの時刻情報を収集して ClockReading を生成する。

        Args:
            node_id: 収集するノードの識別子

        Returns:
            生成した ClockReading
        """
        # 壁時計ナノ秒を取得する
        import time as time_module
        wall_ns = int(time_module.time_ns())
        # HLC タイムスタンプを計算する（壁時計ミリ秒部分を使用する）
        hlc_wall_ms = wall_ns // 1_000_000
        # 論理カウンタは 0 固定（単一ノードのローカル収集では競合しない）
        hlc_logical = 0
        # chrony データを取得する
        chrony_data = self._verifier.read_chrony_data(node_id)
        # chrony の誤差をナノ秒に変換する
        chrony_offset_ns = -1
        ntp_stratum = -1
        if chrony_data is not None:
            chrony_offset_ns = int(abs(chrony_data.rms_offset_s) * 1_000_000_000)
            ntp_stratum = chrony_data.stratum
        # ClockReading を構築する
        reading = ClockReading(
            node_id=node_id,
            wall_clock_ns=wall_ns,
            hlc_wall_ms=hlc_wall_ms,
            hlc_logical=hlc_logical,
            source="local",
            chrony_rms_offset_ns=chrony_offset_ns,
            ntp_stratum=ntp_stratum,
        )
        # 収集した読み取り値を HLCVerifier に投入する
        self._verifier.add_reading(reading)
        # 収集済みノード数カウンタをインクリメントする
        self._collected_nodes += 1
        return reading

    def collect_from_remote(self, node_id: str, remote_ip: str) -> ClockReading | None:
        """リモートノードの時刻情報を SSH 経由で収集する（dry_run では模擬）。

        Args:
            node_id: リモートノードの識別子
            remote_ip: リモートノードの IP アドレス

        Returns:
            生成した ClockReading、失敗時は None
        """
        # dry-run モードの場合はダミーデータを生成する
        if self._dry_run:
            import time as time_module
            import random
            # ローカルと少し異なるタイムスタンプを生成して分散環境を模擬する
            offset_ns = random.randint(-200_000, 200_000)
            wall_ns = int(time_module.time_ns()) + offset_ns
            reading = ClockReading(
                node_id=node_id,
                wall_clock_ns=wall_ns,
                hlc_wall_ms=wall_ns // 1_000_000,
                hlc_logical=0,
                source="simulated_remote",
                chrony_rms_offset_ns=50_000,
                ntp_stratum=3,
            )
            self._verifier.add_reading(reading)
            self._collected_nodes += 1
            return reading
        # 実際の SSH 収集は環境依存のため省略してログのみ記録する
        logger.warning(
            "ClockReadingCollector: SSH 収集は未実装 node=%s ip=%s", node_id, remote_ip
        )
        return None

    @property
    def collected_nodes(self) -> int:
        """収集済みノード数を返す。"""
        return self._collected_nodes
