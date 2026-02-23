from __future__ import annotations

import asyncio

from .exceptions import BulkheadFullError


class Bulkhead:
    def __init__(self, max_concurrent: int, max_wait: float) -> None:
        self._semaphore = asyncio.Semaphore(max_concurrent)
        self._max_concurrent = max_concurrent
        self._max_wait = max_wait

    async def acquire(self) -> None:
        try:
            await asyncio.wait_for(self._semaphore.acquire(), timeout=self._max_wait)
        except asyncio.TimeoutError:
            raise BulkheadFullError(self._max_concurrent)

    def release(self) -> None:
        self._semaphore.release()
