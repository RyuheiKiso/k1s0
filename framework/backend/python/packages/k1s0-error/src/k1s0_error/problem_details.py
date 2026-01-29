"""RFC 7807 Problem Details representation."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from k1s0_error.exception import K1s0Exception


_STATUS_TITLES: dict[int, str] = {
    400: "Bad Request",
    401: "Unauthorized",
    403: "Forbidden",
    404: "Not Found",
    409: "Conflict",
    500: "Internal Server Error",
}


@dataclass(frozen=True)
class ProblemDetails:
    """RFC 7807 Problem Details response body.

    Attributes:
        status: HTTP status code.
        title: Short human-readable summary of the problem type.
        detail: Human-readable explanation specific to this occurrence.
        error_code: k1s0 structured error code.
        trace_id: Distributed trace ID for correlation.
    """

    status: int
    title: str
    detail: str
    error_code: str
    trace_id: str | None = None

    @classmethod
    def from_exception(cls, exc: K1s0Exception) -> ProblemDetails:
        """Create ProblemDetails from a K1s0Exception."""
        title = _STATUS_TITLES.get(exc.http_status, "Error")
        return cls(
            status=exc.http_status,
            title=title,
            detail=exc.detail,
            error_code=str(exc.error_code),
            trace_id=exc.trace_id,
        )

    def to_dict(self) -> dict[str, object]:
        """Serialize to a dictionary suitable for JSON responses."""
        result: dict[str, object] = {
            "status": self.status,
            "title": self.title,
            "detail": self.detail,
            "error_code": self.error_code,
        }
        if self.trace_id is not None:
            result["trace_id"] = self.trace_id
        return result
