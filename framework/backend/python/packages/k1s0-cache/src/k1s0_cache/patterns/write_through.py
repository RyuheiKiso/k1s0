"""Write-through caching pattern."""

from __future__ import annotations

from collections.abc import Awaitable, Callable

from k1s0_cache.operations import CacheOperations


class WriteThrough:
    """Writes to the backing store first, then updates the cache.

    Args:
        cache: Cache operations instance.
        default_ttl: Default TTL applied when none is specified per call.
    """

    def __init__(
        self, cache: CacheOperations, default_ttl: int | None = None
    ) -> None:
        self._cache = cache
        self._default_ttl = default_ttl

    async def write(
        self,
        key: str,
        value: str,
        writer: Callable[[str, str], Awaitable[None]],
        ttl: int | None = None,
    ) -> None:
        """Write to the backing store and then update the cache.

        Args:
            key: The cache key.
            value: The value to write.
            writer: An async callable that persists the key-value pair.
            ttl: Optional TTL override.
        """
        await writer(key, value)
        effective_ttl = ttl if ttl is not None else self._default_ttl
        await self._cache.set(key, value, ttl=effective_ttl)
