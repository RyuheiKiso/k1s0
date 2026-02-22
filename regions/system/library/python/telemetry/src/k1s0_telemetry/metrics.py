"""OpenTelemetry REDメトリクス定義"""

from __future__ import annotations

from opentelemetry import metrics

_meter = metrics.get_meter("k1s0", version="0.1.0")

request_total = _meter.create_counter(
    name="request_total",
    description="Total number of requests",
    unit="1",
)

request_duration_seconds = _meter.create_histogram(
    name="request_duration_seconds",
    description="Request duration in seconds",
    unit="s",
)

request_errors_total = _meter.create_counter(
    name="request_errors_total",
    description="Total number of request errors",
    unit="1",
)

requests_in_flight = _meter.create_up_down_counter(
    name="requests_in_flight",
    description="Number of requests currently being processed",
    unit="1",
)
