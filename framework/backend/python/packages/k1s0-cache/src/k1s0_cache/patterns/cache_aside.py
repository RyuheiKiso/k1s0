"""Cache-aside (lazy-loading) pattern."""

from __future__ import annotations

from collections.abc import Awaitable, Callable

from k1s0_cache.operations import CacheOperations


class CacheAside:
    """Implements the cache-aside (lazy-loading) pattern.

    On a cache miss the provided loader is called, and its result is stored
    in the cache before being returned.

    Args:
        cache: Cache operations instance.
        default_ttl: Default TTL applied when none is specified per call.
    """

    def __init__(
        self, cache: CacheOperations, default_ttl: int | None = None
    ) -> None:
        self._cache = cache
        self._default_ttl = default_ttl

    async def get_or_load(
        self,
        key: str,
        loader: Callable[[], Awaitable[str]],
        ttl: int | None = None,
    ) -> str:
        """Return the cached value, or load and cache it on a miss.

        Args:
            key: The cache key.
            loader: An async callable that produces the value on a miss.
            ttl: Optional TTL override.

        Returns:
            The cached or freshly-loaded value.
        """
        value = await self._cache.get(key)
        if value is not None:
            return value

        value = await loader()
        effective_ttl = ttl if ttl is not None else self._default_ttl
        await self._cache.set(key, value, ttl=effective_ttl)
        return value
