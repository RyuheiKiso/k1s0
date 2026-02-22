"""init_telemetry のユニットテスト"""

from k1s0_telemetry.initializer import init_telemetry
from k1s0_telemetry.models import TelemetryConfig, TraceConfig


def test_init_telemetry_with_defaults() -> None:
    """デフォルト設定で初期化できること。"""
    config = TelemetryConfig(service_name="test-service")
    # 例外が発生しないことを確認
    init_telemetry(config)


def test_init_telemetry_with_trace_disabled() -> None:
    """トレース無効設定で初期化できること。"""
    config = TelemetryConfig(
        service_name="test-service",
        trace=TraceConfig(enabled=False),
    )
    init_telemetry(config)


def test_init_telemetry_with_trace_enabled_no_endpoint() -> None:
    """エンドポイントなしでトレース有効設定の初期化。"""
    config = TelemetryConfig(
        service_name="test-service",
        trace=TraceConfig(enabled=True, sample_rate=0.5),
    )
    init_telemetry(config)
