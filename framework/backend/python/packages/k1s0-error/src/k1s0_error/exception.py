"""k1s0 exception hierarchy."""

from __future__ import annotations

from k1s0_error.error_code import ErrorCode


class K1s0Exception(Exception):
    """Base exception for all k1s0 services.

    Attributes:
        error_code: Structured error code in {service}.{category}.{reason} format.
        http_status: HTTP status code for REST responses.
        trace_id: Optional distributed trace ID for correlation.
        detail: Human-readable error detail message.
    """

    def __init__(
        self,
        error_code: str | ErrorCode,
        detail: str,
        http_status: int = 500,
        trace_id: str | None = None,
    ) -> None:
        super().__init__(detail)
        self.error_code = ErrorCode(error_code) if isinstance(error_code, str) else error_code
        self.http_status = http_status
        self.trace_id = trace_id
        self.detail = detail


class NotFoundException(K1s0Exception):
    """Resource not found (HTTP 404)."""

    def __init__(
        self,
        error_code: str | ErrorCode,
        detail: str,
        trace_id: str | None = None,
    ) -> None:
        super().__init__(error_code=error_code, detail=detail, http_status=404, trace_id=trace_id)


class ValidationException(K1s0Exception):
    """Validation failure (HTTP 400)."""

    def __init__(
        self,
        error_code: str | ErrorCode,
        detail: str,
        trace_id: str | None = None,
    ) -> None:
        super().__init__(error_code=error_code, detail=detail, http_status=400, trace_id=trace_id)


class ConflictException(K1s0Exception):
    """Resource conflict (HTTP 409)."""

    def __init__(
        self,
        error_code: str | ErrorCode,
        detail: str,
        trace_id: str | None = None,
    ) -> None:
        super().__init__(error_code=error_code, detail=detail, http_status=409, trace_id=trace_id)


class UnauthorizedException(K1s0Exception):
    """Authentication required (HTTP 401)."""

    def __init__(
        self,
        error_code: str | ErrorCode,
        detail: str,
        trace_id: str | None = None,
    ) -> None:
        super().__init__(error_code=error_code, detail=detail, http_status=401, trace_id=trace_id)


class ForbiddenException(K1s0Exception):
    """Insufficient permissions (HTTP 403)."""

    def __init__(
        self,
        error_code: str | ErrorCode,
        detail: str,
        trace_id: str | None = None,
    ) -> None:
        super().__init__(error_code=error_code, detail=detail, http_status=403, trace_id=trace_id)
