"""Rate-limit-specific error hierarchy."""

from __future__ import annotations

from datetime import timedelta


class RateLimitError(Exception):
    """Base exception for all rate limiting operations.

    Attributes:
        error_code: Structured error code identifying the failure.
        is_retryable: Whether the caller should retry the operation.
    """

    def __init__(
        self,
        error_code: str,
        detail: str,
        *,
        is_retryable: bool = False,
    ) -> None:
        super().__init__(detail)
        self.error_code = error_code
        self.is_retryable = is_retryable
        self.detail = detail


class RateLimitExceededError(RateLimitError):
    """Rate limit has been exceeded and the request was rejected.

    This error is retryable because the caller can wait until
    the rate limit window resets or tokens are refilled.

    Attributes:
        retry_after: Duration until the next request may succeed.
    """

    def __init__(
        self,
        retry_after: timedelta,
        detail: str = "Rate limit exceeded",
    ) -> None:
        super().__init__(
            error_code="rate_limit.exceeded",
            detail=detail,
            is_retryable=True,
        )
        self.retry_after = retry_after
