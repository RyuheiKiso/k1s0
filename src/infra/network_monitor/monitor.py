"""src/infra/network_monitor/monitor.py

ネットワークジッタおよびレイテンシー監視モジュール。
仕様: 12_クラスタ位相適合仕様.md §topology_class

- JitterSample: タイムスタンプ・送受信元・RTT・ジッタを保持する
- NetworkMonitor: ICMP ping を使ったプローブ・ジッタ計算・輻輳検知を提供する
- TopologyProbe: k8s pod 間のレイテンシーマトリクスを発見する
- 結果は時系列 JSONL ファイル（1 時間ごと）に保存する
- p99 RTT または ジッタが閾値を超えた場合にアラートを発行する
"""

from __future__ import annotations

import json
import logging
import math
import os
import subprocess
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

# モジュールロガーを初期化する
logger = logging.getLogger(__name__)

# デフォルトの p99 RTT アラート閾値（マイクロ秒）: 50 ms
_DEFAULT_P99_RTT_THRESHOLD_US = 50_000
# デフォルトのジッタアラート閾値（マイクロ秒）: 10 ms
_DEFAULT_JITTER_THRESHOLD_US = 10_000
# JSONL ファイルの保存先ディレクトリ
_DEFAULT_STORAGE_DIR = Path(os.environ.get("NETWORK_MONITOR_STORAGE", "/tmp/k1s0_network_monitor"))
# ping の応答タイムアウト（秒）
_PING_TIMEOUT_S = 5
# デフォルトの ping 回数
_DEFAULT_PING_COUNT = 10
# pod-to-pod 探索に使用する名前空間
_K8S_NAMESPACE = os.environ.get("K8S_NAMESPACE", "k1s0")


# ---------------------------------------------------------------------------
# JitterSample データクラス
# ---------------------------------------------------------------------------

@dataclass
class JitterSample:
    """1 回のネットワーク計測サンプルを保持するデータクラス。"""

    # サンプル取得時刻（Unix タイムスタンプ、ナノ秒精度）
    timestamp_ns: int
    # 送信元アドレスまたは識別子
    src: str
    # 送信先アドレスまたは識別子
    dst: str
    # 往復遅延時間（RTT, マイクロ秒）
    rtt_us: int
    # 前サンプルとの遅延変動（ジッタ, マイクロ秒、最初のサンプルは 0）
    jitter_us: int

    def to_dict(self) -> dict[str, Any]:
        """JitterSample を辞書形式にシリアライズする。"""
        return {
            "timestamp_ns": self.timestamp_ns,
            "src": self.src,
            "dst": self.dst,
            "rtt_us": self.rtt_us,
            "jitter_us": self.jitter_us,
        }


# ---------------------------------------------------------------------------
# JitterStats データクラス
# ---------------------------------------------------------------------------

@dataclass
class JitterStats:
    """ジッタ統計情報を保持するデータクラス。"""

    # サンプル数
    sample_count: int
    # 平均 RTT（マイクロ秒）
    mean_rtt_us: float
    # 最小 RTT（マイクロ秒）
    min_rtt_us: int
    # 最大 RTT（マイクロ秒）
    max_rtt_us: int
    # p50 RTT（中央値、マイクロ秒）
    p50_rtt_us: int
    # p95 RTT（マイクロ秒）
    p95_rtt_us: int
    # p99 RTT（マイクロ秒）
    p99_rtt_us: int
    # 平均ジッタ（マイクロ秒）
    mean_jitter_us: float
    # 最大ジッタ（マイクロ秒）
    max_jitter_us: int
    # 標準偏差（マイクロ秒）
    std_dev_us: float

    def to_dict(self) -> dict[str, Any]:
        """JitterStats を辞書形式にシリアライズする。"""
        return {
            "sample_count": self.sample_count,
            "mean_rtt_us": self.mean_rtt_us,
            "min_rtt_us": self.min_rtt_us,
            "max_rtt_us": self.max_rtt_us,
            "p50_rtt_us": self.p50_rtt_us,
            "p95_rtt_us": self.p95_rtt_us,
            "p99_rtt_us": self.p99_rtt_us,
            "mean_jitter_us": self.mean_jitter_us,
            "max_jitter_us": self.max_jitter_us,
            "std_dev_us": self.std_dev_us,
        }


# ---------------------------------------------------------------------------
# LatencyMatrix データクラス
# ---------------------------------------------------------------------------

@dataclass
class LatencyMatrix:
    """pod 間のレイテンシーマトリクスを保持するデータクラス。"""

    # 測定対象の pod/node 名リスト
    endpoints: list[str]
    # src_endpoint → dst_endpoint → JitterStats のマッピング
    matrix: dict[str, dict[str, JitterStats]]
    # 測定実行時刻（Unix タイムスタンプ）
    measured_at: float = field(default_factory=time.time)

    def get_stats(self, src: str, dst: str) -> JitterStats | None:
        """指定ペアの JitterStats を返す。存在しない場合は None を返す。"""
        # ネストした辞書から値を取得する
        return self.matrix.get(src, {}).get(dst)

    def max_p99_us(self) -> int:
        """マトリクス全体の最大 p99 RTT をマイクロ秒で返す。"""
        # 全エントリの p99_rtt_us の最大値を求める
        max_val = 0
        for src_data in self.matrix.values():
            for stats in src_data.values():
                if stats.p99_rtt_us > max_val:
                    max_val = stats.p99_rtt_us
        return max_val

    def to_dict(self) -> dict[str, Any]:
        """LatencyMatrix を辞書形式にシリアライズする。"""
        # マトリクスを辞書に変換する
        matrix_dict: dict[str, dict[str, Any]] = {}
        for src, dst_map in self.matrix.items():
            matrix_dict[src] = {dst: stats.to_dict() for dst, stats in dst_map.items()}
        return {
            "endpoints": self.endpoints,
            "matrix": matrix_dict,
            "measured_at": self.measured_at,
        }


# ---------------------------------------------------------------------------
# CongestionResult データクラス
# ---------------------------------------------------------------------------

@dataclass
class CongestionResult:
    """輻輳検知の結果を保持するデータクラス。"""

    # 輻輳が検知されたか
    congested: bool
    # 輻輳の根拠となった指標
    reason: str
    # 輻輳度合い（0.0〜1.0、高いほど深刻）
    severity: float
    # アラートを発行すべきか
    should_alert: bool

    def to_dict(self) -> dict[str, Any]:
        """CongestionResult を辞書形式にシリアライズする。"""
        return {
            "congested": self.congested,
            "reason": self.reason,
            "severity": self.severity,
            "should_alert": self.should_alert,
        }


# ---------------------------------------------------------------------------
# PingParser ユーティリティクラス
# ---------------------------------------------------------------------------

class PingParser:
    """ping コマンドの出力をパースするユーティリティクラス。"""

    @staticmethod
    def parse_rtt_ms(line: str) -> float | None:
        """ping 出力の 1 行から RTT ミリ秒を抽出する。

        "64 bytes from ...: icmp_seq=1 ttl=64 time=1.23 ms" のような行を処理する。

        Args:
            line: ping 出力の 1 行

        Returns:
            RTT ミリ秒（float）、パース失敗の場合は None
        """
        # "time=" の後の数値を抽出する
        if "time=" not in line:
            return None
        try:
            # "time=" 以降の部分を取得する
            after_time = line.split("time=")[1]
            # 最初の空白で分割して数値部分を取得する
            rtt_str = after_time.split()[0]
            return float(rtt_str)
        except (IndexError, ValueError):
            return None

    @staticmethod
    def ms_to_us(rtt_ms: float) -> int:
        """ミリ秒をマイクロ秒に変換する。"""
        # 1 ms = 1000 us
        return int(rtt_ms * 1_000)


# ---------------------------------------------------------------------------
# NetworkMonitor クラス
# ---------------------------------------------------------------------------

class NetworkMonitor:
    """ICMP ping を使ってネットワークジッタとレイテンシーを監視するクラス。"""

    def __init__(
        self,
        p99_threshold_us: int = _DEFAULT_P99_RTT_THRESHOLD_US,
        jitter_threshold_us: int = _DEFAULT_JITTER_THRESHOLD_US,
        storage_dir: Path | None = None,
        dry_run: bool = True,
    ) -> None:
        """NetworkMonitor を初期化する。

        Args:
            p99_threshold_us: p99 RTT アラート閾値（マイクロ秒）
            jitter_threshold_us: ジッタアラート閾値（マイクロ秒）
            storage_dir: JSONL ファイル保存先ディレクトリ
            dry_run: True の場合は実際の ping を実行しない
        """
        # p99 RTT アラート閾値を保持する
        self._p99_threshold_us = p99_threshold_us
        # ジッタアラート閾値を保持する
        self._jitter_threshold_us = jitter_threshold_us
        # JSONL 保存先ディレクトリを設定する
        self._storage_dir = storage_dir or _DEFAULT_STORAGE_DIR
        # dry-run モードフラグを保持する
        self._dry_run = dry_run
        # 保存先ディレクトリが存在しない場合は作成する
        self._storage_dir.mkdir(parents=True, exist_ok=True)
        # アラートコールバックのリストを初期化する
        self._alert_callbacks: list[Any] = []

    def probe(self, target: str, count: int = _DEFAULT_PING_COUNT) -> list[JitterSample]:
        """指定ターゲットに ICMP ping を送信してジッタサンプルを収集する。

        Args:
            target: ping 送信先のホスト名または IP アドレス
            count: 送信する ping の回数

        Returns:
            収集した JitterSample のリスト
        """
        # dry-run モードの場合はダミーサンプルを返す
        if self._dry_run:
            return self._generate_dummy_samples(target, count)
        # ping コマンドを実行する
        rtt_us_list = self._run_ping(target, count)
        # JitterSample リストを構築する
        samples = self._build_samples(target, rtt_us_list)
        # 結果を JSONL ファイルに保存する
        self._save_samples(samples)
        return samples

    def _run_ping(self, target: str, count: int) -> list[int]:
        """ping コマンドを実行して RTT リスト（マイクロ秒）を返す。"""
        # ping コマンド引数を構築する
        cmd = ["ping", "-c", str(count), "-W", str(_PING_TIMEOUT_S), target]
        # ping を実行する
        try:
            result = subprocess.run(
                cmd, capture_output=True, text=True, timeout=_PING_TIMEOUT_S * count + 5
            )
        except subprocess.TimeoutExpired:
            logger.error("NetworkMonitor: ping タイムアウト target=%s", target)
            return []
        except FileNotFoundError:
            logger.error("NetworkMonitor: ping コマンドが見つからない")
            return []
        # 標準出力から RTT を抽出する
        rtt_us_list: list[int] = []
        for line in result.stdout.splitlines():
            rtt_ms = PingParser.parse_rtt_ms(line)
            if rtt_ms is not None:
                rtt_us_list.append(PingParser.ms_to_us(rtt_ms))
        return rtt_us_list

    def _generate_dummy_samples(self, target: str, count: int) -> list[JitterSample]:
        """dry-run モード用のダミーサンプルを生成する。"""
        # ベース RTT を 1 ms〜5 ms の範囲で設定する
        import random
        base_rtt_us = random.randint(1_000, 5_000)
        samples: list[JitterSample] = []
        prev_rtt_us = base_rtt_us
        for i in range(count):
            # ノイズを加えてリアルな RTT 変動を模擬する
            noise_us = random.randint(-200, 200)
            rtt_us = max(100, base_rtt_us + noise_us)
            jitter_us = abs(rtt_us - prev_rtt_us) if i > 0 else 0
            ts_ns = int(time.time() * 1_000_000_000) + i * 1_000_000
            samples.append(JitterSample(
                timestamp_ns=ts_ns,
                src="localhost",
                dst=target,
                rtt_us=rtt_us,
                jitter_us=jitter_us,
            ))
            prev_rtt_us = rtt_us
        return samples

    def _build_samples(self, target: str, rtt_us_list: list[int]) -> list[JitterSample]:
        """RTT リストから JitterSample リストを構築する。"""
        # サンプルリストを初期化する
        samples: list[JitterSample] = []
        prev_rtt_us = rtt_us_list[0] if rtt_us_list else 0
        for i, rtt_us in enumerate(rtt_us_list):
            # ジッタを計算する（前サンプルとの差分の絶対値）
            jitter_us = abs(rtt_us - prev_rtt_us) if i > 0 else 0
            ts_ns = int(time.time() * 1_000_000_000)
            samples.append(JitterSample(
                timestamp_ns=ts_ns,
                src="local",
                dst=target,
                rtt_us=rtt_us,
                jitter_us=jitter_us,
            ))
            prev_rtt_us = rtt_us
        return samples

    def measure_jitter(self, samples: list[JitterSample]) -> JitterStats:
        """サンプルリストから JitterStats を計算する。

        Args:
            samples: 計算対象の JitterSample リスト

        Returns:
            JitterStats
        """
        # サンプルが空の場合はゼロ値の統計を返す
        if not samples:
            return JitterStats(
                sample_count=0,
                mean_rtt_us=0.0,
                min_rtt_us=0,
                max_rtt_us=0,
                p50_rtt_us=0,
                p95_rtt_us=0,
                p99_rtt_us=0,
                mean_jitter_us=0.0,
                max_jitter_us=0,
                std_dev_us=0.0,
            )
        # RTT リストをソートしてパーセンタイルを計算する
        rtt_values = sorted(s.rtt_us for s in samples)
        n = len(rtt_values)
        # 平均 RTT を計算する
        mean_rtt = sum(rtt_values) / n
        # 標準偏差を計算する
        variance = sum((r - mean_rtt) ** 2 for r in rtt_values) / n
        std_dev = math.sqrt(variance)
        # パーセンタイルを計算する補助関数
        def percentile(sorted_vals: list[int], p: float) -> int:
            # インデックスを計算して値を返す
            idx = max(0, int(math.ceil(p / 100.0 * len(sorted_vals))) - 1)
            return sorted_vals[idx]
        # ジッタ統計を計算する
        jitter_values = [s.jitter_us for s in samples if s.jitter_us > 0]
        mean_jitter = sum(jitter_values) / len(jitter_values) if jitter_values else 0.0
        max_jitter = max(jitter_values) if jitter_values else 0
        return JitterStats(
            sample_count=n,
            mean_rtt_us=mean_rtt,
            min_rtt_us=rtt_values[0],
            max_rtt_us=rtt_values[-1],
            p50_rtt_us=percentile(rtt_values, 50),
            p95_rtt_us=percentile(rtt_values, 95),
            p99_rtt_us=percentile(rtt_values, 99),
            mean_jitter_us=mean_jitter,
            max_jitter_us=max_jitter,
            std_dev_us=std_dev,
        )

    def detect_congestion(self, samples: list[JitterSample]) -> CongestionResult:
        """サンプルリストから輻輳を検知する。

        p99 RTT が閾値を超えているか、またはジッタが閾値を超えている場合に
        輻輳と判定する。

        Args:
            samples: 判定対象の JitterSample リスト

        Returns:
            CongestionResult
        """
        # サンプルが空の場合は輻輳なしと判定する
        if not samples:
            return CongestionResult(
                congested=False, reason="no_samples", severity=0.0, should_alert=False
            )
        # 統計を計算する
        stats = self.measure_jitter(samples)
        # p99 RTT が閾値を超えているか確認する
        p99_exceeds = stats.p99_rtt_us > self._p99_threshold_us
        # ジッタが閾値を超えているか確認する
        jitter_exceeds = stats.max_jitter_us > self._jitter_threshold_us
        # 輻輳判定フラグを計算する
        congested = p99_exceeds or jitter_exceeds
        # 輻輳理由を構築する
        reasons = []
        if p99_exceeds:
            reasons.append(
                f"p99_rtt={stats.p99_rtt_us}us > threshold={self._p99_threshold_us}us"
            )
        if jitter_exceeds:
            reasons.append(
                f"max_jitter={stats.max_jitter_us}us > threshold={self._jitter_threshold_us}us"
            )
        reason = "; ".join(reasons) if reasons else "within_limits"
        # 輻輳度合いを計算する（p99 超過率と jitter 超過率の最大値）
        p99_ratio = stats.p99_rtt_us / self._p99_threshold_us if self._p99_threshold_us > 0 else 0.0
        jitter_ratio = stats.max_jitter_us / self._jitter_threshold_us if self._jitter_threshold_us > 0 else 0.0
        severity = min(1.0, max(p99_ratio, jitter_ratio))
        # アラート発行判定（severity > 1.0 = 閾値超過）
        should_alert = congested
        # 輻輳検知結果をログに記録する
        if congested:
            logger.warning("NetworkMonitor: 輻輳検知 reason=%s severity=%.2f", reason, severity)
        return CongestionResult(
            congested=congested, reason=reason, severity=severity, should_alert=should_alert
        )

    def register_alert_callback(self, callback: Any) -> None:
        """アラート発行時に呼び出すコールバックを登録する。"""
        # コールバックリストに追加する
        self._alert_callbacks.append(callback)

    def _fire_alert(self, result: CongestionResult, target: str) -> None:
        """輻輳アラートを発行して全コールバックを呼び出す。"""
        # コールバックが登録されていない場合はスキップする
        if not self._alert_callbacks:
            logger.warning("NetworkMonitor: アラート発行 target=%s reason=%s", target, result.reason)
            return
        # 全コールバックを呼び出す
        for callback in self._alert_callbacks:
            try:
                callback(target, result)
            except Exception as exc:
                logger.error("NetworkMonitor: アラートコールバックエラー error=%s", exc)

    def _save_samples(self, samples: list[JitterSample]) -> None:
        """サンプルを時系列 JSONL ファイルに追記保存する。"""
        # 現在時刻から時間単位のファイル名を生成する
        import datetime
        now = datetime.datetime.utcnow()
        filename = now.strftime("network_%Y%m%d_%H.jsonl")
        filepath = self._storage_dir / filename
        # JSONL 形式で追記する
        try:
            with open(filepath, "a", encoding="utf-8") as fh:
                for sample in samples:
                    fh.write(json.dumps(sample.to_dict()) + "\n")
        except Exception as exc:
            logger.error("NetworkMonitor: JSONL 保存エラー path=%s error=%s", filepath, exc)

    def probe_and_alert(self, target: str, count: int = _DEFAULT_PING_COUNT) -> tuple[list[JitterSample], CongestionResult]:
        """プローブを実行して輻輳検知しアラートを発行する統合メソッド。

        Args:
            target: プローブ対象
            count: ping 回数

        Returns:
            (JitterSample リスト, CongestionResult) のタプル
        """
        # プローブを実行する
        samples = self.probe(target, count)
        # 輻輳を検知する
        result = self.detect_congestion(samples)
        # アラートが必要な場合は発行する
        if result.should_alert:
            self._fire_alert(result, target)
        return samples, result


# ---------------------------------------------------------------------------
# TopologyProbe クラス
# ---------------------------------------------------------------------------

class TopologyProbe:
    """k8s クラスター内の pod 間レイテンシーマトリクスを発見するクラス。"""

    def __init__(
        self,
        monitor: NetworkMonitor,
        namespace: str = _K8S_NAMESPACE,
        dry_run: bool = True,
    ) -> None:
        """TopologyProbe を初期化する。

        Args:
            monitor: 使用する NetworkMonitor インスタンス
            namespace: 対象 k8s 名前空間
            dry_run: True の場合は kubectl 実行をスキップする
        """
        # NetworkMonitor インスタンスを保持する
        self._monitor = monitor
        # 対象名前空間を保持する
        self._namespace = namespace
        # dry-run モードフラグを保持する
        self._dry_run = dry_run
        # 発見した pod 名 → IP アドレスのマッピングを初期化する
        self._pod_ips: dict[str, str] = {}

    def discover_pods(self) -> dict[str, str]:
        """名前空間内の pod 名 → IP アドレスのマッピングを取得する。

        dry-run モードの場合はダミーデータを返す。

        Returns:
            pod 名 → IP アドレスの辞書
        """
        # dry-run モードの場合はダミーデータを返す
        if self._dry_run:
            self._pod_ips = {
                "k1s0-tier1-0": "10.0.0.1",
                "k1s0-tier1-1": "10.0.0.2",
                "k1s0-tier2-0": "10.0.1.1",
                "k1s0-tier2-1": "10.0.1.2",
                "k1s0-ops-0": "10.0.2.1",
            }
            return self._pod_ips
        # kubectl で pod リストを取得する
        try:
            result = subprocess.run(
                [
                    "kubectl", "get", "pods",
                    "-n", self._namespace,
                    "-o", "jsonpath={range .items[*]}{.metadata.name} {.status.podIP}\\n{end}",
                ],
                capture_output=True, text=True, timeout=30
            )
            # 出力をパースして辞書を構築する
            pod_ips: dict[str, str] = {}
            for line in result.stdout.strip().splitlines():
                parts = line.strip().split()
                if len(parts) == 2:
                    pod_name, pod_ip = parts
                    pod_ips[pod_name] = pod_ip
            self._pod_ips = pod_ips
            return pod_ips
        except Exception as exc:
            logger.error("TopologyProbe: kubectl 実行エラー error=%s", exc)
            return {}

    def measure_latency_matrix(self, count: int = 5) -> LatencyMatrix:
        """全 pod ペア間のレイテンシーマトリクスを計測する。

        Args:
            count: 各ペアの ping 回数

        Returns:
            LatencyMatrix
        """
        # pod リストを取得する
        if not self._pod_ips:
            self.discover_pods()
        # エンドポイントリストを取得する
        endpoints = list(self._pod_ips.keys())
        # マトリクス辞書を初期化する
        matrix: dict[str, dict[str, JitterStats]] = {}
        # 全ペアを計測する（src ≠ dst）
        for src_pod in endpoints:
            matrix[src_pod] = {}
            for dst_pod in endpoints:
                # 同一 pod 間の計測はスキップする
                if src_pod == dst_pod:
                    continue
                # 送信先 IP アドレスを取得する
                dst_ip = self._pod_ips.get(dst_pod, dst_pod)
                # プローブを実行してジッタを計算する
                samples = self._monitor.probe(dst_ip, count)
                stats = self._monitor.measure_jitter(samples)
                matrix[src_pod][dst_pod] = stats
                logger.debug(
                    "TopologyProbe: 計測完了 src=%s dst=%s p99=%d us",
                    src_pod, dst_pod, stats.p99_rtt_us
                )
        return LatencyMatrix(
            endpoints=endpoints,
            matrix=matrix,
            measured_at=time.time(),
        )

    def find_high_latency_pairs(
        self,
        matrix: LatencyMatrix,
        threshold_us: int = _DEFAULT_P99_RTT_THRESHOLD_US,
    ) -> list[tuple[str, str, int]]:
        """p99 RTT が閾値を超えている pod ペアを抽出する。

        Args:
            matrix: 検索対象の LatencyMatrix
            threshold_us: p99 RTT の閾値（マイクロ秒）

        Returns:
            (src, dst, p99_rtt_us) のタプルリスト（p99 降順）
        """
        # 閾値超過ペアを収集する
        high_latency_pairs: list[tuple[str, str, int]] = []
        for src, dst_map in matrix.matrix.items():
            for dst, stats in dst_map.items():
                if stats.p99_rtt_us > threshold_us:
                    high_latency_pairs.append((src, dst, stats.p99_rtt_us))
        # p99 降順にソートする
        high_latency_pairs.sort(key=lambda x: x[2], reverse=True)
        return high_latency_pairs

    def save_matrix(self, matrix: LatencyMatrix, output_path: Path | None = None) -> Path:
        """レイテンシーマトリクスを JSON ファイルに保存する。

        Args:
            matrix: 保存する LatencyMatrix
            output_path: 保存先パス（None の場合はデフォルトパスを使用する）

        Returns:
            保存したファイルのパス
        """
        # 保存先パスを決定する
        if output_path is None:
            ts_int = int(matrix.measured_at)
            filename = f"latency_matrix_{ts_int}.json"
            output_path = _DEFAULT_STORAGE_DIR / filename
        # 保存先ディレクトリが存在しない場合は作成する
        output_path.parent.mkdir(parents=True, exist_ok=True)
        # JSON 形式で保存する
        with open(output_path, "w", encoding="utf-8") as fh:
            json.dump(matrix.to_dict(), fh, ensure_ascii=False, indent=2)
        logger.info("TopologyProbe: マトリクス保存 path=%s", output_path)
        return output_path


# ---------------------------------------------------------------------------
# NetworkHealthReport データクラス
# ---------------------------------------------------------------------------

@dataclass
class NetworkHealthReport:
    """ネットワーク健全性の総合レポートを保持するデータクラス。"""

    # レポート生成時刻
    generated_at: float
    # 対象ターゲットリスト
    targets: list[str]
    # ターゲットごとのジッタ統計情報
    stats_per_target: dict[str, JitterStats]
    # ターゲットごとの輻輳検知結果
    congestion_per_target: dict[str, CongestionResult]
    # 全体の健全性判定（全ターゲットが正常の場合 True）
    overall_healthy: bool
    # アラートを発行すべきターゲットのリスト
    alerting_targets: list[str]

    def to_dict(self) -> dict[str, Any]:
        """NetworkHealthReport を辞書形式にシリアライズする。"""
        # 全フィールドを辞書にマッピングする
        return {
            "generated_at": self.generated_at,
            "targets": self.targets,
            "stats_per_target": {t: s.to_dict() for t, s in self.stats_per_target.items()},
            "congestion_per_target": {t: c.to_dict() for t, c in self.congestion_per_target.items()},
            "overall_healthy": self.overall_healthy,
            "alerting_targets": self.alerting_targets,
        }


# ---------------------------------------------------------------------------
# MultiTargetMonitor クラス
# ---------------------------------------------------------------------------

class MultiTargetMonitor:
    """複数のターゲットを同時に監視してネットワーク健全性レポートを生成するクラス。"""

    def __init__(
        self,
        monitor: NetworkMonitor,
        targets: list[str],
        probe_count: int = _DEFAULT_PING_COUNT,
    ) -> None:
        """MultiTargetMonitor を初期化する。

        Args:
            monitor: 使用する NetworkMonitor インスタンス
            targets: 監視対象のターゲットリスト
            probe_count: 各ターゲットの ping 回数
        """
        # NetworkMonitor インスタンスを保持する
        self._monitor = monitor
        # 監視対象ターゲットリストを保持する
        self._targets = targets
        # ping 回数を保持する
        self._probe_count = probe_count
        # 統計履歴（ターゲット → JitterStats のリスト）を初期化する
        self._stats_history: dict[str, list[JitterStats]] = {t: [] for t in targets}

    def run_all(self) -> NetworkHealthReport:
        """全ターゲットへのプローブを実行してネットワーク健全性レポートを返す。

        Returns:
            NetworkHealthReport
        """
        # ターゲットごとの統計と輻輳結果を収集する
        stats_per_target: dict[str, JitterStats] = {}
        congestion_per_target: dict[str, CongestionResult] = {}
        alerting_targets: list[str] = []
        for target in self._targets:
            # プローブを実行して輻輳を検知する
            samples, congestion = self._monitor.probe_and_alert(target, self._probe_count)
            # ジッタ統計を計算する
            stats = self._monitor.measure_jitter(samples)
            stats_per_target[target] = stats
            congestion_per_target[target] = congestion
            # 統計履歴に追加する
            self._stats_history[target].append(stats)
            # アラートが必要なターゲットを記録する
            if congestion.should_alert:
                alerting_targets.append(target)
        # 全体の健全性を判定する
        overall_healthy = len(alerting_targets) == 0
        return NetworkHealthReport(
            generated_at=time.time(),
            targets=self._targets,
            stats_per_target=stats_per_target,
            congestion_per_target=congestion_per_target,
            overall_healthy=overall_healthy,
            alerting_targets=alerting_targets,
        )

    def get_trend(self, target: str, window: int = 5) -> dict[str, Any]:
        """指定ターゲットの直近 window 件の統計トレンドを返す。

        Args:
            target: トレンドを取得するターゲット
            window: 直近何件の統計を使用するか

        Returns:
            トレンド情報を含む辞書
        """
        # 対象ターゲットの履歴を取得する
        history = self._stats_history.get(target, [])
        # 直近 window 件を取得する
        recent = history[-window:] if len(history) >= window else history
        # 履歴が空の場合は空辞書を返す
        if not recent:
            return {"target": target, "insufficient_data": True}
        # 平均 p99 RTT トレンドを計算する
        avg_p99 = sum(s.p99_rtt_us for s in recent) / len(recent)
        # p99 の最大値と最小値を計算する
        max_p99 = max(s.p99_rtt_us for s in recent)
        min_p99 = min(s.p99_rtt_us for s in recent)
        # 平均ジッタトレンドを計算する
        avg_jitter = sum(s.mean_jitter_us for s in recent) / len(recent)
        return {
            "target": target,
            "sample_count": len(recent),
            "avg_p99_rtt_us": avg_p99,
            "max_p99_rtt_us": max_p99,
            "min_p99_rtt_us": min_p99,
            "avg_jitter_us": avg_jitter,
        }

    def save_report(self, report: NetworkHealthReport, storage_dir: Path | None = None) -> Path:
        """ネットワーク健全性レポートを JSON ファイルに保存する。

        Args:
            report: 保存するレポート
            storage_dir: 保存先ディレクトリ

        Returns:
            保存したファイルのパス
        """
        # 保存先を決定する
        target_dir = storage_dir or _DEFAULT_STORAGE_DIR
        target_dir.mkdir(parents=True, exist_ok=True)
        ts_int = int(report.generated_at)
        filename = f"network_health_{ts_int}.json"
        filepath = target_dir / filename
        # JSON 形式で保存する
        with open(filepath, "w", encoding="utf-8") as fh:
            json.dump(report.to_dict(), fh, ensure_ascii=False, indent=2)
        logger.info("MultiTargetMonitor: レポート保存 path=%s", filepath)
        return filepath


# ---------------------------------------------------------------------------
# PacketLossDetector クラス
# ---------------------------------------------------------------------------

class PacketLossDetector:
    """パケットロスを検出して記録するクラス。"""

    def __init__(self, loss_threshold_pct: float = 1.0) -> None:
        """PacketLossDetector を初期化する。

        Args:
            loss_threshold_pct: アラートを発行するパケットロス率の閾値（%）
        """
        # パケットロス率の閾値を保持する
        self._loss_threshold_pct = loss_threshold_pct
        # ターゲットごとのロス率履歴を初期化する
        self._loss_history: dict[str, list[float]] = {}

    def measure_loss(self, target: str, sent: int, received: int) -> float:
        """パケットロス率を計算して履歴に記録する。

        Args:
            target: 計測対象のターゲット
            sent: 送信したパケット数
            received: 受信したパケット数

        Returns:
            パケットロス率（%）
        """
        # 送信数が 0 の場合はロスなしとみなす
        if sent == 0:
            return 0.0
        # ロス率を計算する
        lost = sent - received
        loss_pct = (lost / sent) * 100.0
        # 履歴に追記する
        if target not in self._loss_history:
            self._loss_history[target] = []
        self._loss_history[target].append(loss_pct)
        return loss_pct

    def is_exceeding_threshold(self, loss_pct: float) -> bool:
        """ロス率が閾値を超えているか確認する。"""
        return loss_pct > self._loss_threshold_pct

    def get_average_loss(self, target: str, window: int = 10) -> float:
        """指定ターゲットの直近 window 件の平均ロス率を返す。"""
        # 履歴を取得する
        history = self._loss_history.get(target, [])
        recent = history[-window:] if len(history) >= window else history
        if not recent:
            return 0.0
        return sum(recent) / len(recent)

    def parse_ping_loss(self, ping_output: str) -> float:
        """ping 出力の統計行からパケットロス率を抽出する。"""
        # "X% packet loss" パターンを探す
        for line in ping_output.splitlines():
            if "packet loss" in line and "%" in line:
                try:
                    # "X%" の部分を抽出する
                    pct_str = line.split("%")[0].split()[-1]
                    return float(pct_str)
                except (IndexError, ValueError):
                    pass
        return 0.0

    def to_dict(self) -> dict[str, Any]:
        """PacketLossDetector の状態を辞書形式で返す。"""
        # ターゲットごとの平均ロス率を集計する
        averages = {target: self.get_average_loss(target) for target in self._loss_history}
        return {
            "loss_threshold_pct": self._loss_threshold_pct,
            "tracked_targets": list(self._loss_history.keys()),
            "average_loss_pct": averages,
        }
