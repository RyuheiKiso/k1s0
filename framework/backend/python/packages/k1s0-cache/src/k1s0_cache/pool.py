"""Redis connection pool management."""

from __future__ import annotations

from typing import Any

from k1s0_cache.config import CacheConfig
from k1s0_cache.errors import ConnectionError

try:
    import redis.asyncio as aioredis

    _HAS_REDIS = True
except ImportError:  # pragma: no cover
    _HAS_REDIS = False


class RedisPool:
    """Manages an async Redis connection pool.

    Args:
        config: Cache configuration.

    Raises:
        ConnectionError: If the ``redis`` package is not installed.
    """

    def __init__(self, config: CacheConfig) -> None:
        if not _HAS_REDIS:
            msg = (
                "The 'redis' package is required. "
                "Install k1s0-cache with the [redis] extra."
            )
            raise ConnectionError(msg)

        self._config = config
        self._pool: Any = aioredis.ConnectionPool(
            host=config.host,
            port=config.port,
            db=config.db,
            max_connections=config.pool_size,
            socket_connect_timeout=config.connect_timeout,
            decode_responses=True,
        )

    async def get_connection(self) -> Any:
        """Acquire a Redis connection from the pool.

        Returns:
            An ``redis.asyncio.Redis`` instance backed by the pool.
        """
        return aioredis.Redis(connection_pool=self._pool)

    async def close(self) -> None:
        """Close all connections in the pool."""
        await self._pool.disconnect()
