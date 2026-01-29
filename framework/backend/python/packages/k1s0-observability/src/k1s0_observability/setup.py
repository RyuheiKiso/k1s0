"""Observability setup for FastAPI applications."""

from __future__ import annotations

import logging
from typing import Any

from opentelemetry import trace
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter
from opentelemetry.sdk.resources import Resource
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor

logger = logging.getLogger("k1s0.observability")


def setup_observability(
    service_name: str,
    otlp_endpoint: str | None = None,
    **kwargs: Any,
) -> TracerProvider:
    """Initialize OpenTelemetry tracing for a k1s0 service.

    Sets up a TracerProvider with OTLP export. Can be integrated with FastAPI
    via OpenTelemetry instrumentation packages.

    Args:
        service_name: Name of the service for resource identification.
        otlp_endpoint: OTLP collector endpoint. Defaults to localhost:4317.
        **kwargs: Additional resource attributes.

    Returns:
        The configured TracerProvider.
    """
    resource = Resource.create(
        {
            "service.name": service_name,
            **kwargs,
        }
    )

    provider = TracerProvider(resource=resource)

    endpoint = otlp_endpoint or "localhost:4317"
    exporter = OTLPSpanExporter(endpoint=endpoint, insecure=True)
    processor = BatchSpanProcessor(exporter)
    provider.add_span_processor(processor)

    trace.set_tracer_provider(provider)

    logger.info("Observability initialized for service '%s' -> %s", service_name, endpoint)
    return provider
