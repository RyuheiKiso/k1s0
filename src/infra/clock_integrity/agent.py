"""src/infra/clock_integrity/agent.py

clock_integrity monitoring agent。
仕様: 13_時刻整合適合仕様.md §clock_integrity_class
- chrony/PTP の tracking データを取得して classes.yaml の max_skew_us と比較する
- 閾値超過時に alert を発行する
- Prometheus テキスト形式でメトリクスを /metrics エンドポイントで公開する
- eBPF clock_skew_probe と連携する（probe がない場合は chronyc で代替）

環境変数:
  CLOCK_DRY_RUN: "true" で実際の chronyc コマンドをスキップ（デフォルト true）
  CLOCK_METRICS_PORT: Prometheus metrics サーバーポート（デフォルト 9094）
  CLOCK_POLL_INTERVAL: ポーリング間隔秒数（デフォルト 10）
"""

from __future__ import annotations

import argparse
import datetime
import http.server
import logging
import os
import re
import subprocess
import sys
import threading
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

try:
    import yaml
    _HAS_YAML = True
except ImportError:
    print("PyYAML required", file=sys.stderr)
    sys.exit(1)

# ロガーを設定する
logger = logging.getLogger(__name__)

# このファイルのあるディレクトリ（src/infra/clock_integrity/）
_CLOCK_DIR = Path(__file__).parent
# classes.yaml のパス（clock_integrity_class 定義の SoT）
_CLASSES_YAML = _CLOCK_DIR / "classes.yaml"
# dry_run デフォルト値を環境変数から取得する
_DRY_RUN_DEFAULT = os.environ.get("CLOCK_DRY_RUN", "true").lower() in ("1", "true", "yes")
# ポーリング間隔秒数
_POLL_INTERVAL = int(os.environ.get("CLOCK_POLL_INTERVAL", "10"))
# Prometheus metrics サーバーポート
_METRICS_PORT = int(os.environ.get("CLOCK_METRICS_PORT", "9094"))


# ---------------------------------------------------------------------------
# clock_integrity_class データクラス
# ---------------------------------------------------------------------------

@dataclass(frozen=True)
class ClockIntegrityClass:
    """clock_integrity の 1 クラス定義。"""
    # クラス識別子（例: v1_intra_rack_ptp）
    class_id: str
    # 時刻同期プロトコル（ptp_ieee1588v2 / ntp_chrony_stratum1 / ntp_chrony_wan 等）
    sync_protocol: str
    # 許容最大スキュー（マイクロ秒）
    max_skew_us: int
    # eBPF monitoring が必須か
    ebpf_monitoring_required: bool


@dataclass
class ClockMeasurement:
    """1 回の時刻スキュー計測結果を保持するクラス。"""
    # 計測したクラス識別子
    class_id: str
    # 計測時刻（UNIX time）
    measured_at: float
    # 計測スキュー（マイクロ秒）
    skew_us: float
    # 計測ソース（chronyc / ptp4l / ebpf / simulated）
    source: str
    # 閾値超過かどうか
    threshold_exceeded: bool
    # 最大許容スキュー
    max_skew_us: int


# ---------------------------------------------------------------------------
# クラス定義ローダー
# ---------------------------------------------------------------------------

def _load_classes(classes_yaml: Path) -> dict[str, ClockIntegrityClass]:
    """classes.yaml から clock_integrity_class 定義をロードする。"""
    # ファイルが存在しない場合はデフォルトを返す
    if not classes_yaml.exists():
        logger.warning("classes.yaml not found: %s, using defaults", classes_yaml)
        return _default_classes()

    # YAML をパースする
    data = yaml.safe_load(classes_yaml.read_text(encoding="utf-8")) or {}
    result: dict[str, ClockIntegrityClass] = {}

    # clock_integrity_classes リストを走査して各クラスをロードする
    for entry in data.get("clock_integrity_classes", []):
        class_id = str(entry.get("class_id", ""))
        if not class_id:
            continue
        result[class_id] = ClockIntegrityClass(
            class_id=class_id,
            sync_protocol=str(entry.get("sync_protocol", "ntp_chrony_stratum1")),
            max_skew_us=int(entry.get("max_skew_us", 1000)),
            ebpf_monitoring_required=bool(entry.get("ebpf_monitoring_required", False)),
        )

    # クラスが空の場合はデフォルトを返す
    return result if result else _default_classes()


def _default_classes() -> dict[str, ClockIntegrityClass]:
    """13_時刻整合適合仕様.md の 5 クラスをデフォルトとして返す。"""
    return {
        "v1_intra_rack_ptp": ClockIntegrityClass(
            "v1_intra_rack_ptp", "ptp_ieee1588v2", 1, True
        ),
        "v1_dc_chrony_stratum1": ClockIntegrityClass(
            "v1_dc_chrony_stratum1", "ntp_chrony_stratum1", 1000, True
        ),
        "v1_multi_region_ntp": ClockIntegrityClass(
            "v1_multi_region_ntp", "ntp_chrony_wan", 50000, False
        ),
        "v1_edge_ptp": ClockIntegrityClass(
            "v1_edge_ptp", "ptp_ieee1588v2", 5, True
        ),
        "v1_ops_edge_ntp": ClockIntegrityClass(
            "v1_ops_edge_ntp", "ntp_chrony_stratum1", 10000, False
        ),
    }


# ---------------------------------------------------------------------------
# スキュー計測エンジン
# ---------------------------------------------------------------------------

class ClockSkewCollector:
    """clock_skew を chronyc / ptp4l / ebpf から取得するコレクタ。"""

    def __init__(
        self,
        classes: dict[str, ClockIntegrityClass],
        dry_run: bool = _DRY_RUN_DEFAULT,
    ) -> None:
        # クラス定義を保持する
        self._classes = classes
        # dry_run フラグを設定する
        self._dry_run = dry_run

    def measure_all(self) -> list[ClockMeasurement]:
        """全クラスのスキューを計測して ClockMeasurement リストを返す。"""
        results: list[ClockMeasurement] = []
        for class_id, cls in self._classes.items():
            measurement = self._measure(cls)
            results.append(measurement)
        return results

    def _measure(self, cls: ClockIntegrityClass) -> ClockMeasurement:
        """1 クラスのスキューを計測する。"""
        # dry_run の場合はシミュレーション値を返す
        if self._dry_run:
            return self._simulate_measurement(cls)

        # プロトコル別に計測方法を選択する
        if cls.sync_protocol.startswith("ptp"):
            skew_us, source = self._measure_ptp(cls)
        else:
            skew_us, source = self._measure_chrony(cls)

        # eBPF monitoring が必須の場合は ebpf 計測を試みる
        if cls.ebpf_monitoring_required:
            ebpf_skew, ebpf_source = self._measure_ebpf(cls)
            if ebpf_source != "unavailable":
                # eBPF 計測が成功した場合は より精度の高い値を採用する
                skew_us = max(skew_us, ebpf_skew)
                source = f"{source}+{ebpf_source}"

        # 閾値超過チェック
        exceeded = (skew_us > cls.max_skew_us)
        return ClockMeasurement(
            class_id=cls.class_id,
            measured_at=time.time(),
            skew_us=skew_us,
            source=source,
            threshold_exceeded=exceeded,
            max_skew_us=cls.max_skew_us,
        )

    def _simulate_measurement(self, cls: ClockIntegrityClass) -> ClockMeasurement:
        """dry_run 用のシミュレーション計測を返す（スキュー=0: 正常状態）。"""
        # WSL2 環境では全ノードが同一ホスト時刻を共有するため skew=0
        simulated_skew_us = 0.0
        return ClockMeasurement(
            class_id=cls.class_id,
            measured_at=time.time(),
            skew_us=simulated_skew_us,
            source="simulated",
            threshold_exceeded=False,
            max_skew_us=cls.max_skew_us,
        )

    def _measure_chrony(self, cls: ClockIntegrityClass) -> tuple[float, str]:
        """chronyc tracking コマンドでシステム時刻オフセットを取得する。"""
        try:
            # chronyc tracking コマンドを実行する
            result = subprocess.run(
                ["chronyc", "tracking"],
                capture_output=True, text=True, timeout=5,
            )
            output = result.stdout
            # "System time" 行から offset 値を抽出する（例: "System time     :   0.000012345 seconds slow"）
            match = re.search(r"System time\s*:\s*([\d.]+)\s*seconds", output)
            if match:
                # 秒をマイクロ秒に変換する
                offset_seconds = float(match.group(1))
                offset_us = offset_seconds * 1_000_000
                return offset_us, "chronyc"
            return 0.0, "chronyc_no_data"
        except (subprocess.TimeoutExpired, FileNotFoundError) as exc:
            logger.debug("chronyc unavailable: %s", exc)
            return 0.0, "chronyc_unavailable"

    def _measure_ptp(self, cls: ClockIntegrityClass) -> tuple[float, str]:
        """ptp4l のログからオフセットを取得する。"""
        try:
            # pmc コマンドで PTP マスターからのオフセットを取得する
            result = subprocess.run(
                ["pmc", "-u", "-b", "0", "GET CURRENT_DATA_SET"],
                capture_output=True, text=True, timeout=5,
            )
            output = result.stdout
            # "offsetFromMaster" 行からオフセット値を抽出する（ナノ秒単位）
            match = re.search(r"offsetFromMaster\s+(-?\d+)", output)
            if match:
                # ナノ秒をマイクロ秒に変換する
                offset_ns = abs(int(match.group(1)))
                offset_us = offset_ns / 1000.0
                return offset_us, "ptp4l"
            return 0.0, "ptp4l_no_data"
        except (subprocess.TimeoutExpired, FileNotFoundError) as exc:
            logger.debug("ptp4l unavailable: %s", exc)
            return 0.0, "ptp4l_unavailable"

    def _measure_ebpf(self, cls: ClockIntegrityClass) -> tuple[float, str]:
        """eBPF clock_skew_probe からスキューデータを取得する。"""
        # eBPF probe の出力ファイルを確認する（/sys/kernel/debug 等）
        probe_output = Path("/tmp/clock_skew_probe.txt")
        if not probe_output.exists():
            return 0.0, "unavailable"
        try:
            # probe 出力ファイルからスキュー値を読み込む
            content = probe_output.read_text(encoding="utf-8")
            match = re.search(r"skew_ns=(-?\d+)", content)
            if match:
                skew_ns = abs(int(match.group(1)))
                # ナノ秒をマイクロ秒に変換する
                skew_us = skew_ns / 1000.0
                return skew_us, "ebpf"
            return 0.0, "ebpf_no_data"
        except OSError as exc:
            logger.debug("ebpf probe read failed: %s", exc)
            return 0.0, "unavailable"


# ---------------------------------------------------------------------------
# Prometheus メトリクスエクスポーター
# ---------------------------------------------------------------------------

class PrometheusExporter:
    """clock_skew メトリクスを Prometheus テキスト形式で公開するクラス。"""

    def __init__(self) -> None:
        # 最新の計測結果を保持する
        self._measurements: list[ClockMeasurement] = []
        # スレッドセーフな更新のためのロック
        self._lock = threading.Lock()

    def update(self, measurements: list[ClockMeasurement]) -> None:
        """計測結果を更新する。"""
        with self._lock:
            self._measurements = list(measurements)

    def render(self) -> str:
        """Prometheus テキスト形式でメトリクスを生成して返す。"""
        with self._lock:
            measurements = list(self._measurements)

        lines: list[str] = []
        # メトリクス名とヘルプテキストを定義する
        lines.append("# HELP k1s0_clock_skew_us clock skew in microseconds per class")
        lines.append("# TYPE k1s0_clock_skew_us gauge")

        for m in measurements:
            # クラス識別子をラベルとして使用する
            lines.append(
                f'k1s0_clock_skew_us{{class_id="{m.class_id}",source="{m.source}"}} {m.skew_us}'
            )

        lines.append("# HELP k1s0_clock_threshold_exceeded 1 if skew exceeds threshold")
        lines.append("# TYPE k1s0_clock_threshold_exceeded gauge")

        for m in measurements:
            # 閾値超過の場合は 1、それ以外は 0 を出力する
            exceeded_val = 1 if m.threshold_exceeded else 0
            lines.append(
                f'k1s0_clock_threshold_exceeded{{class_id="{m.class_id}"}} {exceeded_val}'
            )

        # 最終更新時刻メトリクスを追加する
        lines.append("# HELP k1s0_clock_last_measured_timestamp last measurement UNIX timestamp")
        lines.append("# TYPE k1s0_clock_last_measured_timestamp gauge")
        for m in measurements:
            lines.append(
                f'k1s0_clock_last_measured_timestamp{{class_id="{m.class_id}"}} {m.measured_at:.3f}'
            )

        return "\n".join(lines) + "\n"


# ---------------------------------------------------------------------------
# HTTP サーバー（/metrics エンドポイント）
# ---------------------------------------------------------------------------

def _make_metrics_handler(exporter: PrometheusExporter) -> type[http.server.BaseHTTPRequestHandler]:
    """Prometheus metrics を提供する HTTP ハンドラクラスを返す。"""
    class MetricsHandler(http.server.BaseHTTPRequestHandler):
        def do_GET(self) -> None:
            if self.path == "/metrics":
                # Prometheus テキスト形式でメトリクスを出力する
                body = exporter.render().encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "text/plain; version=0.0.4; charset=utf-8")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
            elif self.path == "/healthz":
                # ヘルスチェックエンドポイント
                body = b'{"status":"ok"}'
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
            else:
                self.send_response(404)
                self.end_headers()

        def log_message(self, format: str, *args: Any) -> None:
            # HTTP アクセスログを通常ログに転送する
            logger.debug(format, *args)

    return MetricsHandler


# ---------------------------------------------------------------------------
# メインエージェントループ
# ---------------------------------------------------------------------------

class ClockIntegrityAgent:
    """clock_integrity monitoring の全体を管理するエージェント。"""

    def __init__(
        self,
        classes_yaml: Path = _CLASSES_YAML,
        dry_run: bool = _DRY_RUN_DEFAULT,
        poll_interval: int = _POLL_INTERVAL,
        metrics_port: int = _METRICS_PORT,
    ) -> None:
        # クラス定義をロードする
        self._classes = _load_classes(classes_yaml)
        # コレクタを初期化する
        self._collector = ClockSkewCollector(self._classes, dry_run=dry_run)
        # メトリクスエクスポーターを初期化する
        self._exporter = PrometheusExporter()
        # ポーリング間隔を設定する
        self._poll_interval = poll_interval
        # metrics サーバーポートを設定する
        self._metrics_port = metrics_port
        # dry_run フラグを保持する
        self._dry_run = dry_run
        # 実行フラグ
        self._running = False

    def start(self) -> None:
        """metrics HTTP サーバーとポーリングループを起動する。"""
        # metrics HTTP サーバーをバックグラウンドスレッドで起動する
        handler_cls = _make_metrics_handler(self._exporter)
        server = http.server.HTTPServer(("0.0.0.0", self._metrics_port), handler_cls)
        server_thread = threading.Thread(target=server.serve_forever, daemon=True)
        server_thread.start()
        logger.info(
            "clock_integrity agent started: metrics=:%d poll=%ds dry_run=%s",
            self._metrics_port, self._poll_interval, self._dry_run,
        )

        self._running = True
        try:
            self._poll_loop()
        except KeyboardInterrupt:
            logger.info("clock_integrity agent shutting down")
        finally:
            server.shutdown()
            self._running = False

    def _poll_loop(self) -> None:
        """ポーリングループ: poll_interval 秒ごとにスキューを計測する。"""
        while self._running:
            # 全クラスのスキューを計測する
            measurements = self._collector.measure_all()
            # メトリクスを更新する
            self._exporter.update(measurements)
            # 閾値超過があればログ出力する
            for m in measurements:
                if m.threshold_exceeded:
                    logger.error(
                        "CLOCK SKEW ALERT: class=%s skew=%.1fus exceeds max=%dus source=%s",
                        m.class_id, m.skew_us, m.max_skew_us, m.source,
                    )
                else:
                    logger.debug(
                        "clock_skew ok: class=%s skew=%.1fus max=%dus",
                        m.class_id, m.skew_us, m.max_skew_us,
                    )
            # 次のポーリングまで待機する
            time.sleep(self._poll_interval)

    def run_once(self) -> list[ClockMeasurement]:
        """1 回だけ計測して結果を返す（CI / テスト用）。"""
        return self._collector.measure_all()


# ---------------------------------------------------------------------------
# エントリポイント
# ---------------------------------------------------------------------------

def main(argv: list[str] | None = None) -> int:
    """clock_integrity agent のメインエントリポイント。"""
    parser = argparse.ArgumentParser(
        description="clock_integrity monitoring agent: chrony/PTP スキューを監視して alert する"
    )
    parser.add_argument(
        "--dry-run", action="store_true", default=_DRY_RUN_DEFAULT,
        help="実際の chronyc コマンドをスキップしてシミュレーションする",
    )
    parser.add_argument(
        "--port", type=int, default=_METRICS_PORT,
        help=f"Prometheus metrics サーバーポート（デフォルト: {_METRICS_PORT}）",
    )
    parser.add_argument(
        "--poll-interval", type=int, default=_POLL_INTERVAL,
        help=f"ポーリング間隔秒数（デフォルト: {_POLL_INTERVAL}）",
    )
    parser.add_argument(
        "--once", action="store_true", default=False,
        help="1 回だけ計測して終了する",
    )
    args = parser.parse_args(argv)

    # ロギングを設定する
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s %(levelname)s %(name)s %(message)s",
    )

    # エージェントを初期化する
    agent = ClockIntegrityAgent(
        dry_run=args.dry_run,
        poll_interval=args.poll_interval,
        metrics_port=args.port,
    )

    if args.once:
        # 1 回だけ計測して結果を表示する
        measurements = agent.run_once()
        print("\n--- clock_integrity measurements ---")
        for m in measurements:
            status = "EXCEEDED" if m.threshold_exceeded else "OK"
            print(
                f"  [{status}] {m.class_id}: skew={m.skew_us:.1f}us "
                f"(max={m.max_skew_us}us, source={m.source})"
            )
        # 閾値超過がある場合は exit code 1 を返す
        if any(m.threshold_exceeded for m in measurements):
            return 1
        return 0

    # エージェントを起動する（Ctrl-C で停止）
    agent.start()
    return 0


if __name__ == "__main__":
    sys.exit(main())
