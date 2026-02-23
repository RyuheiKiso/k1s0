"""InMemoryCacheClient 実装"""

from __future__ import annotations

import time

from .client import CacheClient


class _CacheEntry:
    __slots__ = ("value", "expires_at")

    def __init__(self, value: str, ttl: float | None) -> None:
        self.value = value
        self.expires_at: float | None = (
            time.monotonic() + ttl if ttl is not None else None
        )

    def is_expired(self) -> bool:
        return self.expires_at is not None and time.monotonic() >= self.expires_at


class InMemoryCacheClient(CacheClient):
    """テスト用インメモリキャッシュクライアント。"""

    def __init__(self) -> None:
        self._store: dict[str, _CacheEntry] = {}

    async def get(self, key: str) -> str | None:
        entry = self._store.get(key)
        if entry is None or entry.is_expired():
            if entry is not None and entry.is_expired():
                del self._store[key]
            return None
        return entry.value

    async def set(self, key: str, value: str, ttl: float | None = None) -> None:
        self._store[key] = _CacheEntry(value, ttl)

    async def delete(self, key: str) -> bool:
        if key in self._store:
            del self._store[key]
            return True
        return False

    async def exists(self, key: str) -> bool:
        return await self.get(key) is not None

    async def set_nx(self, key: str, value: str, ttl: float) -> bool:
        if await self.get(key) is not None:
            return False
        self._store[key] = _CacheEntry(value, ttl)
        return True
