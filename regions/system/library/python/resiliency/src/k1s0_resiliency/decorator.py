from __future__ import annotations

import asyncio
import time
from enum import Enum
from typing import Any, Awaitable, Callable, TypeVar

from .bulkhead import Bulkhead
from .exceptions import (
    CircuitBreakerOpenError,
    MaxRetriesExceededError,
    TimeoutError,
)
from .policy import ResiliencyPolicy

T = TypeVar("T")


class _CircuitState(Enum):
    CLOSED = "closed"
    OPEN = "open"
    HALF_OPEN = "half_open"


class ResiliencyDecorator:
    def __init__(self, policy: ResiliencyPolicy) -> None:
        self._policy = policy
        self._bulkhead: Bulkhead | None = None
        if policy.bulkhead is not None:
            self._bulkhead = Bulkhead(
                max_concurrent=policy.bulkhead.max_concurrent_calls,
                max_wait=policy.bulkhead.max_wait_duration,
            )
        self._cb_state = _CircuitState.CLOSED
        self._cb_failure_count = 0
        self._cb_success_count = 0
        self._cb_last_failure_time: float | None = None

    async def execute(self, fn: Callable[..., Awaitable[T]], *args: Any, **kwargs: Any) -> T:
        self._check_circuit_breaker()

        if self._bulkhead is not None:
            await self._bulkhead.acquire()

        try:
            return await self._execute_with_retry(fn, *args, **kwargs)
        finally:
            if self._bulkhead is not None:
                self._bulkhead.release()

    async def _execute_with_retry(self, fn: Callable[..., Awaitable[T]], *args: Any, **kwargs: Any) -> T:
        max_attempts = self._policy.retry.max_attempts if self._policy.retry else 1
        last_error: BaseException | None = None

        for attempt in range(max_attempts):
            try:
                result = await self._execute_with_timeout(fn, *args, **kwargs)
                self._record_success()
                return result
            except (MaxRetriesExceededError, CircuitBreakerOpenError, TimeoutError):
                raise
            except Exception as e:
                self._record_failure()
                last_error = e
                self._check_circuit_breaker()

                if attempt + 1 < max_attempts and self._policy.retry:
                    delay = _calculate_backoff(
                        attempt,
                        self._policy.retry.base_delay,
                        self._policy.retry.max_delay,
                    )
                    await asyncio.sleep(delay)

        raise MaxRetriesExceededError(max_attempts, last_error)

    async def _execute_with_timeout(self, fn: Callable[..., Awaitable[T]], *args: Any, **kwargs: Any) -> T:
        if self._policy.timeout is None:
            return await fn(*args, **kwargs)

        try:
            return await asyncio.wait_for(fn(*args, **kwargs), timeout=self._policy.timeout)
        except asyncio.TimeoutError:
            raise TimeoutError(self._policy.timeout)

    def _check_circuit_breaker(self) -> None:
        if self._policy.circuit_breaker is None:
            return

        cfg = self._policy.circuit_breaker

        if self._cb_state == _CircuitState.CLOSED:
            return
        elif self._cb_state == _CircuitState.OPEN:
            if self._cb_last_failure_time is not None:
                elapsed = time.monotonic() - self._cb_last_failure_time
                if elapsed >= cfg.recovery_timeout:
                    self._cb_state = _CircuitState.HALF_OPEN
                    self._cb_success_count = 0
                    return
                remaining = cfg.recovery_timeout - elapsed
                raise CircuitBreakerOpenError(remaining)
        # HALF_OPEN: allow through

    def _record_success(self) -> None:
        if self._policy.circuit_breaker is None:
            return

        if self._cb_state == _CircuitState.HALF_OPEN:
            self._cb_success_count += 1
            if self._cb_success_count >= self._policy.circuit_breaker.half_open_max_calls:
                self._cb_state = _CircuitState.CLOSED
                self._cb_failure_count = 0
        elif self._cb_state == _CircuitState.CLOSED:
            self._cb_failure_count = 0

    def _record_failure(self) -> None:
        if self._policy.circuit_breaker is None:
            return

        self._cb_failure_count += 1
        if self._cb_failure_count >= self._policy.circuit_breaker.failure_threshold:
            self._cb_state = _CircuitState.OPEN
            self._cb_last_failure_time = time.monotonic()


async def with_resiliency(policy: ResiliencyPolicy, fn: Callable[..., Awaitable[T]], *args: Any, **kwargs: Any) -> T:
    decorator = ResiliencyDecorator(policy)
    return await decorator.execute(fn, *args, **kwargs)


def _calculate_backoff(attempt: int, base_delay: float, max_delay: float) -> float:
    delay = base_delay * (2 ** attempt)
    return min(delay, max_delay)
