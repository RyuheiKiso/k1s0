"""REDメトリクス定義のユニットテスト"""

from k1s0_telemetry import (
    request_duration_seconds,
    request_errors_total,
    request_total,
    requests_in_flight,
)


def test_metrics_are_not_none() -> None:
    """全メトリクスが None でないこと。"""
    assert request_total is not None
    assert request_duration_seconds is not None
    assert request_errors_total is not None
    assert requests_in_flight is not None


def test_request_total_recordable() -> None:
    """request_total カウンターに記録できること。"""
    # NoopMeterProvider を使用（デフォルト）
    request_total.add(1, {"method": "GET", "path": "/health"})


def test_request_duration_recordable() -> None:
    """request_duration_seconds ヒストグラムに記録できること。"""
    request_duration_seconds.record(0.123, {"method": "GET"})


def test_request_errors_recordable() -> None:
    """request_errors_total カウンターに記録できること。"""
    request_errors_total.add(1, {"error_type": "timeout"})


def test_requests_in_flight_recordable() -> None:
    """requests_in_flight UpDownCounter に記録できること。"""
    requests_in_flight.add(1)
    requests_in_flight.add(-1)
