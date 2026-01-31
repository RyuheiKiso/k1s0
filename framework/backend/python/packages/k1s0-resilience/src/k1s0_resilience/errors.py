"""Resilience-specific error hierarchy."""

from __future__ import annotations


class ResilienceError(Exception):
    """Base exception for all resilience operations.

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


class TimeoutError(ResilienceError):
    """Operation exceeded the configured timeout.

    This error is retryable because the operation may succeed
    if given more time or if the downstream service recovers.
    """

    def __init__(self, detail: str = "Operation timed out") -> None:
        super().__init__(
            error_code="resilience.timeout",
            detail=detail,
            is_retryable=True,
        )


class CircuitOpenError(ResilienceError):
    """Circuit breaker is in OPEN state and rejecting calls.

    This error is not retryable because the circuit breaker has
    determined that the downstream service is unhealthy.
    """

    def __init__(self, detail: str = "Circuit breaker is open") -> None:
        super().__init__(
            error_code="resilience.circuit_open",
            detail=detail,
            is_retryable=False,
        )


class ConcurrencyLimitError(ResilienceError):
    """Concurrency limit has been reached and the call was rejected.

    This error is not retryable because the system is at capacity.
    """

    def __init__(self, detail: str = "Concurrency limit exceeded") -> None:
        super().__init__(
            error_code="resilience.concurrency_limit",
            detail=detail,
            is_retryable=False,
        )
