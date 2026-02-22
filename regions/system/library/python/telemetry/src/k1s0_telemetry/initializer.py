"""OpenTelemetry SDK 初期化"""

from __future__ import annotations

from opentelemetry import trace
from opentelemetry.sdk.resources import Resource
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.sampling import TraceIdRatioBased

from .exceptions import TelemetryError
from .models import TelemetryConfig


def init_telemetry(config: TelemetryConfig) -> None:
    """OpenTelemetry SDK を初期化する。

    Args:
        config: テレメトリー設定

    Raises:
        TelemetryError: 初期化に失敗した場合
    """
    try:
        resource = Resource.create(
            {
                "service.name": config.service_name,
                "service.version": config.service_version,
            }
        )

        if config.trace.enabled:
            sampler = TraceIdRatioBased(config.trace.sample_rate)
            provider = TracerProvider(resource=resource, sampler=sampler)

            if config.trace.endpoint:
                try:
                    from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import (
                        OTLPSpanExporter,
                    )
                    from opentelemetry.sdk.trace.export import BatchSpanProcessor

                    exporter = OTLPSpanExporter(endpoint=config.trace.endpoint)
                    provider.add_span_processor(BatchSpanProcessor(exporter))
                except Exception as e:
                    raise TelemetryError(
                        code="EXPORTER_INIT_ERROR",
                        message=f"Failed to initialize OTLP exporter: {e}",
                        cause=e,
                    ) from e

            trace.set_tracer_provider(provider)
    except TelemetryError:
        raise
    except Exception as e:
        raise TelemetryError(
            code="INIT_ERROR",
            message=f"Failed to initialize telemetry: {e}",
            cause=e,
        ) from e
