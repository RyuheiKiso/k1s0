"""Circuit breaker pattern implementation."""

from __future__ import annotations

import time
from dataclasses import dataclass
from enum import Enum
from typing import Any, Callable


class CircuitState(str, Enum):
    """Circuit breaker states."""

    CLOSED = "closed"
    OPEN = "open"
    HALF_OPEN = "half_open"


@dataclass
class CircuitBreakerConfig:
    """Circuit breaker configuration."""

    failure_threshold: int = 5
    success_threshold: int = 2
    timeout: float = 30.0


class CircuitBreakerError(Exception):
    """Raised when the circuit breaker is open."""


class CircuitBreaker:
    """Circuit breaker for protecting calls to external services."""

    def __init__(self, config: CircuitBreakerConfig | None = None) -> None:
        self._config = config or CircuitBreakerConfig()
        self._state = CircuitState.CLOSED
        self._failure_count = 0
        self._success_count = 0
        self._opened_at: float | None = None

    @property
    def state(self) -> CircuitState:
        """Current circuit state (may transition from OPEN to HALF_OPEN)."""
        if self._state == CircuitState.OPEN and self._is_timeout_expired():
            self._state = CircuitState.HALF_OPEN
            self._success_count = 0
        return self._state

    def record_success(self) -> None:
        """Record a successful call."""
        if self._state == CircuitState.HALF_OPEN:
            self._success_count += 1
            if self._success_count >= self._config.success_threshold:
                self._state = CircuitState.CLOSED
                self._failure_count = 0
                self._success_count = 0
        else:
            self._failure_count = 0

    def record_failure(self) -> None:
        """Record a failed call."""
        self._failure_count += 1
        if self._state == CircuitState.HALF_OPEN:
            self._state = CircuitState.OPEN
            self._opened_at = time.monotonic()
        elif self._failure_count >= self._config.failure_threshold:
            self._state = CircuitState.OPEN
            self._opened_at = time.monotonic()

    async def call(self, fn: Callable[[], Any]) -> Any:
        """Execute a function through the circuit breaker."""
        if self.state == CircuitState.OPEN:
            raise CircuitBreakerError("Circuit breaker is open")
        try:
            result = await fn()
            self.record_success()
            return result
        except Exception:
            self.record_failure()
            raise

    def _is_timeout_expired(self) -> bool:
        if self._opened_at is None:
            return False
        return (time.monotonic() - self._opened_at) >= self._config.timeout
