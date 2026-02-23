"""Trace context for W3C Trace Context propagation."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass
class TraceContext:
    """W3C Trace Context."""

    trace_id: str  # 32 hex chars
    parent_id: str  # 16 hex chars
    flags: int = 1

    def to_traceparent(self) -> str:
        return f"00-{self.trace_id}-{self.parent_id}-{self.flags:02x}"

    @classmethod
    def from_traceparent(cls, s: str) -> TraceContext | None:
        parts = s.split("-")
        if len(parts) != 4 or parts[0] != "00":
            return None
        if len(parts[1]) != 32 or len(parts[2]) != 16 or len(parts[3]) != 2:
            return None
        try:
            flags = int(parts[3], 16)
        except ValueError:
            return None
        return cls(trace_id=parts[1], parent_id=parts[2], flags=flags)
