"""Circuit breaker pattern implementation."""

from __future__ import annotations

import time
from collections.abc import Callable
from dataclasses import dataclass, field
from enum import Enum
from typing import Awaitable, TypeVar

from k1s0_resilience.errors import CircuitOpenError

T = TypeVar("T")


class CircuitState(Enum):
    """Possible states of a circuit breaker."""

    CLOSED = "closed"
    OPEN = "open"
    HALF_OPEN = "half_open"


@dataclass(frozen=True)
class CircuitBreakerConfig:
    """Configuration for the circuit breaker.

    Attributes:
        failure_threshold: Number of failures before opening the circuit.
        success_threshold: Number of successes in HALF_OPEN before closing.
        reset_timeout: Seconds to wait in OPEN before transitioning to HALF_OPEN.
        failure_predicate: Optional predicate to determine if an exception
            counts as a failure. If ``None``, all exceptions are failures.
    """

    failure_threshold: int = 5
    success_threshold: int = 3
    reset_timeout: float = 60.0
    failure_predicate: Callable[[Exception], bool] | None = None


class CircuitBreaker:
    """Circuit breaker that prevents calls to an unhealthy downstream service.

    State machine:
        CLOSED -- failure_threshold reached --> OPEN
        OPEN -- reset_timeout elapsed --> HALF_OPEN
        HALF_OPEN -- success_threshold reached --> CLOSED
        HALF_OPEN -- any failure --> OPEN

    Args:
        config: Circuit breaker configuration.

    Example::

        cb = CircuitBreaker(CircuitBreakerConfig(failure_threshold=3))
        result = await cb.execute(call_downstream())
    """

    def __init__(self, config: CircuitBreakerConfig) -> None:
        self._config = config
        self._state = CircuitState.CLOSED
        self._failure_count = 0
        self._success_count = 0
        self._last_failure_time: float = 0.0
        self._rejected_count = 0
        self._state_transition_count = 0

    @property
    def state(self) -> CircuitState:
        """Return the current circuit state, evaluating timeout transitions."""
        if (
            self._state == CircuitState.OPEN
            and time.monotonic() - self._last_failure_time >= self._config.reset_timeout
        ):
            self._transition_to(CircuitState.HALF_OPEN)
        return self._state

    @property
    def rejected_count(self) -> int:
        """Total number of rejected calls while circuit was OPEN."""
        return self._rejected_count

    @property
    def state_transition_count(self) -> int:
        """Total number of state transitions."""
        return self._state_transition_count

    def _transition_to(self, new_state: CircuitState) -> None:
        """Transition to a new state and reset counters."""
        self._state = new_state
        self._state_transition_count += 1
        if new_state == CircuitState.CLOSED:
            self._failure_count = 0
            self._success_count = 0
        elif new_state == CircuitState.HALF_OPEN:
            self._success_count = 0
        elif new_state == CircuitState.OPEN:
            self._success_count = 0

    def _is_failure(self, exc: Exception) -> bool:
        """Determine whether an exception counts as a circuit-breaker failure."""
        if self._config.failure_predicate is not None:
            return self._config.failure_predicate(exc)
        return True

    async def execute(self, coro: Awaitable[T]) -> T:
        """Execute an awaitable through the circuit breaker.

        Args:
            coro: The awaitable to execute.

        Returns:
            The result of the awaitable.

        Raises:
            CircuitOpenError: If the circuit is OPEN and not yet ready to test.
        """
        current_state = self.state  # triggers timeout check

        if current_state == CircuitState.OPEN:
            self._rejected_count += 1
            raise CircuitOpenError()

        try:
            result = await coro  # type: ignore[misc]
        except Exception as exc:
            if self._is_failure(exc):
                self._record_failure()
            raise

        self._record_success(current_state)
        return result

    def _record_failure(self) -> None:
        """Record a failure and potentially open the circuit."""
        self._failure_count += 1
        self._last_failure_time = time.monotonic()

        if self._state == CircuitState.HALF_OPEN:
            self._transition_to(CircuitState.OPEN)
        elif (
            self._state == CircuitState.CLOSED
            and self._failure_count >= self._config.failure_threshold
        ):
            self._transition_to(CircuitState.OPEN)

    def _record_success(self, state_at_call: CircuitState) -> None:
        """Record a success and potentially close the circuit."""
        if state_at_call == CircuitState.HALF_OPEN:
            self._success_count += 1
            if self._success_count >= self._config.success_threshold:
                self._transition_to(CircuitState.CLOSED)
