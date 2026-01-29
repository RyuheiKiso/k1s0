"""Tests for observability setup."""

from __future__ import annotations

from opentelemetry.sdk.trace import TracerProvider

from k1s0_observability.setup import setup_observability
from k1s0_observability.tracing import get_tracer


class TestSetupObservability:
    def test_returns_tracer_provider(self) -> None:
        provider = setup_observability("test-service")
        assert isinstance(provider, TracerProvider)
        provider.shutdown()

    def test_get_tracer(self) -> None:
        provider = setup_observability("test-service")
        tracer = get_tracer("my-component")
        assert tracer is not None
        provider.shutdown()
