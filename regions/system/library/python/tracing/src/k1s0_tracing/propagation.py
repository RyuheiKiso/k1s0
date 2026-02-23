"""Context propagation helpers."""

from __future__ import annotations

from .baggage import Baggage
from .trace_context import TraceContext


def inject_context(
    headers: dict[str, str],
    ctx: TraceContext,
    baggage: Baggage | None = None,
) -> None:
    headers["traceparent"] = ctx.to_traceparent()
    if baggage is not None:
        header = baggage.to_header()
        if header:
            headers["baggage"] = header


def extract_context(
    headers: dict[str, str],
) -> tuple[TraceContext | None, Baggage]:
    ctx = TraceContext.from_traceparent(headers.get("traceparent", ""))
    baggage = Baggage.from_header(headers.get("baggage", ""))
    return ctx, baggage
