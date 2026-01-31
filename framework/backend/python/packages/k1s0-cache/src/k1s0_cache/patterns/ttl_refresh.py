"""TTL refresh pattern."""

from __future__ import annotations

from k1s0_cache.operations import CacheOperations


class TtlRefresh:
    """Refreshes the TTL of a key on every read.

    Args:
        cache: Cache operations instance.
        refresh_ttl: The TTL (in seconds) to set on each access.
    """

    def __init__(self, cache: CacheOperations, refresh_ttl: int) -> None:
        self._cache = cache
        self._refresh_ttl = refresh_ttl

    async def get_and_refresh(self, key: str) -> str | None:
        """Get a value and refresh its TTL if it exists.

        Args:
            key: The cache key.

        Returns:
            The cached value, or ``None`` if the key does not exist.
        """
        value = await self._cache.get(key)
        if value is not None:
            await self._cache.set(key, value, ttl=self._refresh_ttl)
        return value
