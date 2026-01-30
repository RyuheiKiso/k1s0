"""Concrete Redis cache client."""

from __future__ import annotations

from k1s0_cache.config import CacheConfig
from k1s0_cache.errors import OperationError
from k1s0_cache.hash_operations import HashOperations
from k1s0_cache.list_operations import ListOperations
from k1s0_cache.operations import CacheOperations
from k1s0_cache.pool import RedisPool
from k1s0_cache.set_operations import SetOperations


class CacheClient(CacheOperations, HashOperations, ListOperations, SetOperations):
    """Redis-backed cache client implementing all operation interfaces.

    Args:
        pool: The Redis connection pool.
        config: Cache configuration.
    """

    def __init__(self, pool: RedisPool, config: CacheConfig) -> None:
        self._pool = pool
        self._config = config

    def _prefixed_key(self, key: str) -> str:
        """Return the key with the configured prefix applied.

        Args:
            key: The raw cache key.

        Returns:
            The prefixed key.
        """
        if self._config.prefix:
            return f"{self._config.prefix}:{key}"
        return key

    # -- CacheOperations --

    async def get(self, key: str) -> str | None:
        """Retrieve a value by key."""
        try:
            conn = await self._pool.get_connection()
            result: str | None = await conn.get(self._prefixed_key(key))
            return result
        except Exception as exc:
            raise OperationError(str(exc)) from exc

    async def set(self, key: str, value: str, ttl: int | None = None) -> None:
        """Store a value under the given key."""
        try:
            conn = await self._pool.get_connection()
            effective_ttl = ttl if ttl is not None else self._config.default_ttl
            await conn.set(self._prefixed_key(key), value, ex=effective_ttl)
        except Exception as exc:
            raise OperationError(str(exc)) from exc

    async def delete(self, key: str) -> bool:
        """Delete a key from the cache."""
        try:
            conn = await self._pool.get_connection()
            result: int = await conn.delete(self._prefixed_key(key))
            return result > 0
        except Exception as exc:
            raise OperationError(str(exc)) from exc

    async def exists(self, key: str) -> bool:
        """Check whether a key exists."""
        try:
            conn = await self._pool.get_connection()
            result: int = await conn.exists(self._prefixed_key(key))
            return result > 0
        except Exception as exc:
            raise OperationError(str(exc)) from exc

    async def incr(self, key: str, amount: int = 1) -> int:
        """Increment a numeric value."""
        try:
            conn = await self._pool.get_connection()
            result: int = await conn.incrby(self._prefixed_key(key), amount)
            return result
        except Exception as exc:
            raise OperationError(str(exc)) from exc

    async def decr(self, key: str, amount: int = 1) -> int:
        """Decrement a numeric value."""
        try:
            conn = await self._pool.get_connection()
            result: int = await conn.decrby(self._prefixed_key(key), amount)
            return result
        except Exception as exc:
            raise OperationError(str(exc)) from exc

    # -- HashOperations --

    async def hget(self, key: str, field: str) -> str | None:
        """Get a single field from a hash."""
        try:
            conn = await self._pool.get_connection()
            result: str | None = await conn.hget(self._prefixed_key(key), field)
            return result
        except Exception as exc:
            raise OperationError(str(exc)) from exc

    async def hset(self, key: str, field: str, value: str) -> None:
        """Set a single field in a hash."""
        try:
            conn = await self._pool.get_connection()
            await conn.hset(self._prefixed_key(key), field, value)
        except Exception as exc:
            raise OperationError(str(exc)) from exc

    async def hdel(self, key: str, field: str) -> bool:
        """Delete a field from a hash."""
        try:
            conn = await self._pool.get_connection()
            result: int = await conn.hdel(self._prefixed_key(key), field)
            return result > 0
        except Exception as exc:
            raise OperationError(str(exc)) from exc

    async def hgetall(self, key: str) -> dict[str, str]:
        """Get all fields and values from a hash."""
        try:
            conn = await self._pool.get_connection()
            result: dict[str, str] = await conn.hgetall(self._prefixed_key(key))
            return result
        except Exception as exc:
            raise OperationError(str(exc)) from exc

    # -- ListOperations --

    async def lpush(self, key: str, value: str) -> int:
        """Push a value to the head of a list."""
        try:
            conn = await self._pool.get_connection()
            result: int = await conn.lpush(self._prefixed_key(key), value)
            return result
        except Exception as exc:
            raise OperationError(str(exc)) from exc

    async def rpush(self, key: str, value: str) -> int:
        """Push a value to the tail of a list."""
        try:
            conn = await self._pool.get_connection()
            result: int = await conn.rpush(self._prefixed_key(key), value)
            return result
        except Exception as exc:
            raise OperationError(str(exc)) from exc

    async def lpop(self, key: str) -> str | None:
        """Remove and return the first element of a list."""
        try:
            conn = await self._pool.get_connection()
            result: str | None = await conn.lpop(self._prefixed_key(key))
            return result
        except Exception as exc:
            raise OperationError(str(exc)) from exc

    async def rpop(self, key: str) -> str | None:
        """Remove and return the last element of a list."""
        try:
            conn = await self._pool.get_connection()
            result: str | None = await conn.rpop(self._prefixed_key(key))
            return result
        except Exception as exc:
            raise OperationError(str(exc)) from exc

    async def lrange(self, key: str, start: int, stop: int) -> list[str]:
        """Return a range of elements from a list."""
        try:
            conn = await self._pool.get_connection()
            result: list[str] = await conn.lrange(
                self._prefixed_key(key), start, stop
            )
            return result
        except Exception as exc:
            raise OperationError(str(exc)) from exc

    # -- SetOperations --

    async def sadd(self, key: str, *members: str) -> int:
        """Add one or more members to a set."""
        try:
            conn = await self._pool.get_connection()
            result: int = await conn.sadd(self._prefixed_key(key), *members)
            return result
        except Exception as exc:
            raise OperationError(str(exc)) from exc

    async def srem(self, key: str, *members: str) -> int:
        """Remove one or more members from a set."""
        try:
            conn = await self._pool.get_connection()
            result: int = await conn.srem(self._prefixed_key(key), *members)
            return result
        except Exception as exc:
            raise OperationError(str(exc)) from exc

    async def smembers(self, key: str) -> set[str]:
        """Return all members of a set."""
        try:
            conn = await self._pool.get_connection()
            result: set[str] = await conn.smembers(self._prefixed_key(key))
            return result
        except Exception as exc:
            raise OperationError(str(exc)) from exc

    async def sismember(self, key: str, member: str) -> bool:
        """Check whether a member exists in a set."""
        try:
            conn = await self._pool.get_connection()
            result: bool = await conn.sismember(self._prefixed_key(key), member)
            return result
        except Exception as exc:
            raise OperationError(str(exc)) from exc
