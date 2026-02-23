"""リトライ実行エンジン"""

from __future__ import annotations

import asyncio
from collections.abc import Awaitable, Callable
from typing import TypeVar

from .exceptions import RetryError
from .models import RetryConfig

T = TypeVar("T")


async def with_retry(config: RetryConfig, fn: Callable[[], Awaitable[T]]) -> T:
    """非同期関数をリトライ付きで実行する。"""
    last_error: Exception | None = None
    for attempt in range(config.max_attempts):
        try:
            return await fn()
        except Exception as e:
            last_error = e
            if attempt + 1 < config.max_attempts:
                await asyncio.sleep(config.compute_delay(attempt))
    raise RetryError(attempts=config.max_attempts, last_error=last_error)
