"""k1s0 telemetry library."""

from .exceptions import TelemetryError, TelemetryErrorCodes
from .initializer import init_telemetry
from .logger import new_logger
from .metrics import (
    request_duration_seconds,
    request_errors_total,
    request_total,
    requests_in_flight,
)
from .models import LogConfig, MetricsConfig, TelemetryConfig, TraceConfig

__all__ = [
    "TelemetryConfig",
    "LogConfig",
    "TraceConfig",
    "MetricsConfig",
    "init_telemetry",
    "new_logger",
    "request_total",
    "request_duration_seconds",
    "request_errors_total",
    "requests_in_flight",
    "TelemetryError",
    "TelemetryErrorCodes",
]
