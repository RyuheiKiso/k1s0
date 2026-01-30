"""Write-behind (write-back) caching pattern."""

from __future__ import annotations

import asyncio
from collections.abc import Awaitable, Callable
from dataclasses import dataclass, field

from k1s0_cache.operations import CacheOperations


@dataclass
class WriteBehindStats:
    """Statistics for write-behind operations."""

    total_writes: int = 0
    total_flushes: int = 0
    total_failures: int = 0


class WriteBehind:
    """Buffers writes and flushes them to a backing store in batches.

    The cache is updated immediately, while the backing store receives
    batched writes at a configurable interval.

    Args:
        cache: Cache operations instance.
        writer: An async callable that persists a batch of key-value pairs.
        batch_size: Maximum batch size per flush.
        flush_interval: Seconds between automatic flushes.
        max_retries: Maximum retry attempts for a failed flush.
    """

    def __init__(
        self,
        cache: CacheOperations,
        writer: Callable[[list[tuple[str, str]]], Awaitable[None]],
        batch_size: int = 100,
        flush_interval: float = 5.0,
        max_retries: int = 3,
    ) -> None:
        self._cache = cache
        self._writer = writer
        self._batch_size = batch_size
        self._flush_interval = flush_interval
        self._max_retries = max_retries
        self._buffer: list[tuple[str, str]] = []
        self._task: asyncio.Task[None] | None = None
        self.stats = WriteBehindStats()

    async def write(self, key: str, value: str) -> None:
        """Write to the cache immediately and buffer for the backing store.

        Args:
            key: The cache key.
            value: The value to write.
        """
        await self._cache.set(key, value)
        self._buffer.append((key, value))
        self.stats.total_writes += 1

    async def flush(self) -> None:
        """Flush the current buffer to the backing store."""
        if not self._buffer:
            return

        batch = self._buffer[: self._batch_size]
        self._buffer = self._buffer[self._batch_size :]

        for attempt in range(self._max_retries):
            try:
                await self._writer(batch)
                self.stats.total_flushes += 1
                return
            except Exception:
                if attempt == self._max_retries - 1:
                    self.stats.total_failures += 1
                    raise

    async def _run(self) -> None:
        """Background loop that flushes periodically."""
        while True:
            await asyncio.sleep(self._flush_interval)
            try:
                await self.flush()
            except Exception:
                pass  # stats already updated in flush

    async def start(self) -> None:
        """Start the background flushing loop."""
        if self._task is None:
            self._task = asyncio.create_task(self._run())

    async def stop(self) -> None:
        """Stop the background flushing loop and perform a final flush."""
        if self._task is not None:
            self._task.cancel()
            try:
                await self._task
            except asyncio.CancelledError:
                pass
            self._task = None

        # Final flush of remaining items
        while self._buffer:
            try:
                await self.flush()
            except Exception:
                break
