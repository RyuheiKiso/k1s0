"""Cache health checker."""

from __future__ import annotations

from k1s0_cache.pool import RedisPool


class CacheHealthChecker:
    """Health checker that verifies Redis connectivity via PING.

    Args:
        pool: The Redis connection pool.
    """

    def __init__(self, pool: RedisPool) -> None:
        self._pool = pool

    async def check(self) -> bool:
        """Ping the Redis server.

        Returns:
            ``True`` if the server responds to PING, ``False`` otherwise.
        """
        try:
            conn = await self._pool.get_connection()
            return bool(await conn.ping())
        except Exception:
            return False
