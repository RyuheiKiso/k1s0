"""Timeout guard for async operations."""

from __future__ import annotations

import asyncio
from dataclasses import dataclass
from typing import Awaitable, TypeVar

from k1s0_resilience.errors import TimeoutError

T = TypeVar("T")

_MIN_DURATION: float = 0.1
_MAX_DURATION: float = 300.0


@dataclass(frozen=True)
class TimeoutConfig:
    """Configuration for timeout operations.

    Attributes:
        duration_seconds: Maximum time to wait before timing out.
            Must be between 0.1 and 300.0 seconds.
    """

    duration_seconds: float = 30.0

    def __post_init__(self) -> None:
        if self.duration_seconds < _MIN_DURATION:
            msg = f"duration_seconds must be >= {_MIN_DURATION}, got {self.duration_seconds}"
            raise ValueError(msg)
        if self.duration_seconds > _MAX_DURATION:
            msg = f"duration_seconds must be <= {_MAX_DURATION}, got {self.duration_seconds}"
            raise ValueError(msg)


class TimeoutGuard:
    """Executes an awaitable with a timeout.

    Args:
        config: Timeout configuration.

    Example::

        guard = TimeoutGuard(TimeoutConfig(duration_seconds=5.0))
        result = await guard.execute(some_async_call())
    """

    def __init__(self, config: TimeoutConfig) -> None:
        self._config = config

    async def execute(self, coro: Awaitable[T]) -> T:
        """Execute an awaitable with the configured timeout.

        Args:
            coro: The awaitable to execute.

        Returns:
            The result of the awaitable.

        Raises:
            TimeoutError: If the operation exceeds the configured timeout.
        """
        try:
            return await asyncio.wait_for(
                coro,  # type: ignore[arg-type]
                timeout=self._config.duration_seconds,
            )
        except asyncio.TimeoutError:
            raise TimeoutError(
                detail=f"Operation timed out after {self._config.duration_seconds}s",
            )
