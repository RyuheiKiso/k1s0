"""Concurrency limiter using asyncio semaphores."""

from __future__ import annotations

import asyncio
from dataclasses import dataclass, field
from typing import Awaitable, TypeVar

from k1s0_resilience.errors import ConcurrencyLimitError

T = TypeVar("T")


@dataclass(frozen=True)
class ConcurrencyConfig:
    """Configuration for concurrency limiting.

    Attributes:
        max_concurrent: Maximum number of concurrent executions allowed.
    """

    max_concurrent: int = 10

    def __post_init__(self) -> None:
        if self.max_concurrent < 1:
            msg = f"max_concurrent must be >= 1, got {self.max_concurrent}"
            raise ValueError(msg)


@dataclass
class ConcurrencyMetrics:
    """Metrics for a concurrency limiter.

    Attributes:
        active_count: Number of currently active executions.
        rejected_count: Total number of rejected executions.
    """

    active_count: int = 0
    rejected_count: int = 0


class ConcurrencyLimiter:
    """Limits concurrent execution of async operations.

    Uses an asyncio semaphore internally. If the semaphore cannot be
    acquired immediately, the call is rejected with ``ConcurrencyLimitError``.

    Args:
        config: Concurrency configuration.

    Example::

        limiter = ConcurrencyLimiter(ConcurrencyConfig(max_concurrent=5))
        result = await limiter.execute(some_async_call())
    """

    def __init__(self, config: ConcurrencyConfig) -> None:
        self._config = config
        self._semaphore = asyncio.Semaphore(config.max_concurrent)
        self._metrics = ConcurrencyMetrics()

    @property
    def metrics(self) -> ConcurrencyMetrics:
        """Return current concurrency metrics."""
        return self._metrics

    async def execute(self, coro: Awaitable[T]) -> T:
        """Execute an awaitable if concurrency limit is not exceeded.

        Args:
            coro: The awaitable to execute.

        Returns:
            The result of the awaitable.

        Raises:
            ConcurrencyLimitError: If the concurrency limit has been reached.
        """
        if not self._semaphore._value:  # noqa: SLF001
            self._metrics.rejected_count += 1
            raise ConcurrencyLimitError(
                detail=f"Concurrency limit of {self._config.max_concurrent} exceeded",
            )

        await self._semaphore.acquire()
        self._metrics.active_count += 1
        try:
            return await coro  # type: ignore[misc]
        finally:
            self._metrics.active_count -= 1
            self._semaphore.release()
