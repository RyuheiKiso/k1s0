"""k1s0 correlation library."""

from .context import CorrelationContext
from .exceptions import CorrelationError, CorrelationErrorCodes
from .generator import generate_correlation_id, generate_trace_id
from .headers import X_CORRELATION_ID, X_REQUEST_ID, X_TRACE_ID
from .propagation import (
    extract_from_headers,
    get_correlation_context,
    inject_into_headers,
    set_correlation_context,
)

__all__ = [
    "CorrelationContext",
    "generate_correlation_id",
    "generate_trace_id",
    "X_CORRELATION_ID",
    "X_TRACE_ID",
    "X_REQUEST_ID",
    "set_correlation_context",
    "get_correlation_context",
    "extract_from_headers",
    "inject_into_headers",
    "CorrelationError",
    "CorrelationErrorCodes",
]
