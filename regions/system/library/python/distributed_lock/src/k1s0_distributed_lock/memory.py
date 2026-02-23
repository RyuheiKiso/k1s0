"""In-memory distributed lock implementation."""

from __future__ import annotations

import asyncio
import time
import uuid

from .client import DistributedLock, LockError, LockGuard


class InMemoryDistributedLock(DistributedLock):
    """In-memory distributed lock for testing."""

    def __init__(self) -> None:
        self._locks: dict[str, tuple[str, float]] = {}  # key -> (token, expires_at)
        self._lock = asyncio.Lock()

    async def acquire(self, key: str, ttl: float) -> LockGuard:
        async with self._lock:
            now = time.monotonic()
            if key in self._locks:
                _, expires_at = self._locks[key]
                if now < expires_at:
                    raise LockError(f"Key already locked: {key}")
            token = str(uuid.uuid4())
            self._locks[key] = (token, now + ttl)
            return LockGuard(key=key, token=token)

    async def release(self, guard: LockGuard) -> None:
        async with self._lock:
            if guard.key not in self._locks:
                raise LockError(f"Lock not found: {guard.key}")
            stored_token, _ = self._locks[guard.key]
            if stored_token != guard.token:
                raise LockError("Token mismatch")
            del self._locks[guard.key]

    async def is_locked(self, key: str) -> bool:
        now = time.monotonic()
        if key not in self._locks:
            return False
        _, expires_at = self._locks[key]
        return now < expires_at
