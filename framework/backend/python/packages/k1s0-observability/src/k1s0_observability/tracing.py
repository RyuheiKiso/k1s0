"""Distributed tracing utilities."""

from __future__ import annotations

from opentelemetry import trace


def get_tracer(name: str) -> trace.Tracer:
    """Get a tracer instance for the given component.

    Args:
        name: Component or module name for the tracer.

    Returns:
        An OpenTelemetry Tracer.
    """
    return trace.get_tracer(name)


def get_current_trace_id() -> str | None:
    """Get the current span's trace ID as a hex string.

    Returns:
        The trace ID hex string, or None if no active span.
    """
    span = trace.get_current_span()
    ctx = span.get_span_context()
    if ctx and ctx.trace_id != 0:
        return format(ctx.trace_id, "032x")
    return None
