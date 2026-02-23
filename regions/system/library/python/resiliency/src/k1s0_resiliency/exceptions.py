from __future__ import annotations


class ResiliencyError(Exception):
    """Base exception for resiliency errors."""


class MaxRetriesExceededError(ResiliencyError):
    """Raised when maximum retry attempts are exhausted."""

    def __init__(self, attempts: int, last_error: BaseException | None = None) -> None:
        self.attempts = attempts
        self.last_error = last_error
        super().__init__(f"Max retries exceeded after {attempts} attempts")


class CircuitBreakerOpenError(ResiliencyError):
    """Raised when the circuit breaker is open."""

    def __init__(self, remaining_seconds: float) -> None:
        self.remaining_seconds = remaining_seconds
        super().__init__(f"Circuit breaker open, remaining: {remaining_seconds:.1f}s")


class BulkheadFullError(ResiliencyError):
    """Raised when the bulkhead is full."""

    def __init__(self, max_concurrent: int) -> None:
        self.max_concurrent = max_concurrent
        super().__init__(f"Bulkhead full, max concurrent: {max_concurrent}")


class TimeoutError(ResiliencyError):
    """Raised when an operation times out."""

    def __init__(self, after_seconds: float) -> None:
        self.after_seconds = after_seconds
        super().__init__(f"Operation timed out after {after_seconds:.1f}s")
