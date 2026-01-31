"""Retry executor with exponential backoff and jitter."""

from __future__ import annotations

import asyncio
import random
from collections.abc import Callable
from dataclasses import dataclass
from typing import Awaitable, TypeVar

T = TypeVar("T")


@dataclass(frozen=True)
class RetryConfig:
    """Configuration for retry behavior.

    Attributes:
        max_attempts: Maximum number of attempts (including the initial call).
        initial_interval: Base delay in seconds before the first retry.
        max_interval: Maximum delay in seconds between retries.
        multiplier: Factor by which the delay increases on each retry.
        jitter_factor: Random jitter as a fraction of the computed delay.
        retryable_checker: Optional predicate to determine if an exception
            is retryable. If ``None``, all exceptions are retried.
    """

    max_attempts: int = 3
    initial_interval: float = 1.0
    max_interval: float = 60.0
    multiplier: float = 2.0
    jitter_factor: float = 0.1
    retryable_checker: Callable[[Exception], bool] | None = None

    def __post_init__(self) -> None:
        if self.max_attempts < 1:
            msg = f"max_attempts must be >= 1, got {self.max_attempts}"
            raise ValueError(msg)


class RetryExecutor:
    """Executes an async operation with configurable retry logic.

    Uses exponential backoff with jitter between retry attempts.

    Args:
        config: Retry configuration.

    Example::

        executor = RetryExecutor(RetryConfig(max_attempts=3))
        result = await executor.execute(lambda: fetch_data())
    """

    def __init__(self, config: RetryConfig) -> None:
        self._config = config

    def _calculate_delay(self, attempt: int) -> float:
        """Calculate the delay before the next retry.

        Args:
            attempt: Zero-based attempt number (0 = first retry).

        Returns:
            Delay in seconds with jitter applied.
        """
        delay = self._config.initial_interval * (self._config.multiplier ** attempt)
        delay = min(delay, self._config.max_interval)
        jitter = random.random() * self._config.jitter_factor * delay  # noqa: S311
        return delay + jitter

    def _is_retryable(self, exc: Exception) -> bool:
        """Determine whether an exception should trigger a retry."""
        if self._config.retryable_checker is not None:
            return self._config.retryable_checker(exc)
        return True

    async def execute(self, coro_factory: Callable[[], Awaitable[T]]) -> T:
        """Execute with retries using the provided coroutine factory.

        Args:
            coro_factory: A callable that returns a new awaitable on each
                invocation. Must create a fresh coroutine per attempt.

        Returns:
            The result of the first successful execution.

        Raises:
            Exception: The last exception if all attempts are exhausted.
        """
        last_exception: Exception | None = None

        for attempt in range(self._config.max_attempts):
            try:
                return await coro_factory()
            except Exception as exc:
                last_exception = exc
                is_last_attempt = attempt == self._config.max_attempts - 1

                if is_last_attempt or not self._is_retryable(exc):
                    raise

                delay = self._calculate_delay(attempt)
                await asyncio.sleep(delay)

        # This should never be reached, but satisfies type checker.
        assert last_exception is not None  # noqa: S101
        raise last_exception
