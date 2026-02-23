"""Rate limit exceptions."""

from __future__ import annotations


class RateLimitError(Exception):
    """Rate limit error."""

    def __init__(
        self,
        message: str,
        code: str = "UNKNOWN",
        retry_after_secs: int | None = None,
    ) -> None:
        super().__init__(message)
        self.code = code
        self.retry_after_secs = retry_after_secs
