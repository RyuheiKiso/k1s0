"""k1s0-observability: OpenTelemetry-based observability."""

from __future__ import annotations

from k1s0_observability.metrics import create_counter, create_histogram
from k1s0_observability.setup import setup_observability
from k1s0_observability.tracing import get_tracer

__all__ = [
    "create_counter",
    "create_histogram",
    "get_tracer",
    "setup_observability",
]
