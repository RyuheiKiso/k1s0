"""Named bulkhead pattern for service isolation."""

from __future__ import annotations

import asyncio
from dataclasses import dataclass
from typing import Awaitable, ClassVar, TypeVar

from k1s0_resilience.errors import ConcurrencyLimitError

T = TypeVar("T")


@dataclass(frozen=True)
class BulkheadConfig:
    """Configuration for a named bulkhead.

    Attributes:
        name: Unique name identifying this bulkhead partition.
        max_concurrent: Maximum number of concurrent executions.
    """

    name: str
    max_concurrent: int = 10

    def __post_init__(self) -> None:
        if self.max_concurrent < 1:
            msg = f"max_concurrent must be >= 1, got {self.max_concurrent}"
            raise ValueError(msg)
        if not self.name:
            msg = "Bulkhead name must not be empty"
            raise ValueError(msg)


class Bulkhead:
    """Named bulkhead for isolating resource consumption.

    Multiple ``Bulkhead`` instances with the same name share the same
    underlying semaphore, ensuring that a single downstream dependency
    cannot exhaust all available concurrency.

    Args:
        config: Bulkhead configuration.

    Example::

        bulkhead = Bulkhead(BulkheadConfig(name="payment-service", max_concurrent=5))
        result = await bulkhead.execute(call_payment_api())
    """

    _instances: ClassVar[dict[str, asyncio.Semaphore]] = {}

    def __init__(self, config: BulkheadConfig) -> None:
        self._config = config
        if config.name not in Bulkhead._instances:
            Bulkhead._instances[config.name] = asyncio.Semaphore(config.max_concurrent)
        self._semaphore = Bulkhead._instances[config.name]

    async def execute(self, coro: Awaitable[T]) -> T:
        """Execute an awaitable within this bulkhead's concurrency limit.

        Args:
            coro: The awaitable to execute.

        Returns:
            The result of the awaitable.

        Raises:
            ConcurrencyLimitError: If the bulkhead's concurrency limit is reached.
        """
        if not self._semaphore._value:  # noqa: SLF001
            raise ConcurrencyLimitError(
                detail=f"Bulkhead '{self._config.name}' concurrency limit of "
                f"{self._config.max_concurrent} exceeded",
            )

        await self._semaphore.acquire()
        try:
            return await coro  # type: ignore[misc]
        finally:
            self._semaphore.release()

    @classmethod
    def reset_all(cls) -> None:
        """Reset all named bulkhead instances. Intended for testing."""
        cls._instances.clear()
