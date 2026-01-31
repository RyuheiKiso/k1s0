"""Distributed lock implementations backed by PostgreSQL and Redis."""

from __future__ import annotations

import asyncio
import logging
import time
import uuid
from abc import ABC, abstractmethod
from dataclasses import dataclass
from types import TracebackType

import asyncpg
import redis.asyncio as aioredis

from k1s0_consensus.config import LockConfig
from k1s0_consensus.error import LockTimeoutError
from k1s0_consensus.metrics import LockMetrics

logger = logging.getLogger("k1s0.consensus.lock")


@dataclass(frozen=True)
class LockGuard:
    """Represents a held distributed lock.

    Use as an async context manager to automatically release the lock
    on scope exit.

    Attributes:
        resource: The locked resource name.
        owner_id: The unique identifier of the lock holder.
        fence_token: Monotonic token for fencing.
        expires_at: Unix timestamp when the lock expires.
    """

    resource: str
    owner_id: str
    fence_token: int
    expires_at: float
    _lock_impl: DistributedLock

    async def __aenter__(self) -> LockGuard:
        return self

    async def __aexit__(
        self,
        exc_type: type[BaseException] | None,
        exc_val: BaseException | None,
        exc_tb: TracebackType | None,
    ) -> None:
        await self._lock_impl.unlock(self)


class DistributedLock(ABC):
    """Abstract interface for distributed locks."""

    @abstractmethod
    async def try_lock(self, resource: str, ttl_ms: int | None = None) -> LockGuard | None:
        """Attempt to acquire a lock without waiting.

        Args:
            resource: The resource to lock.
            ttl_ms: Time-to-live in milliseconds. Uses default if None.

        Returns:
            A LockGuard if acquired, None otherwise.
        """

    @abstractmethod
    async def lock(self, resource: str, ttl_ms: int | None = None, timeout_ms: int | None = None) -> LockGuard:
        """Acquire a lock, retrying until success or timeout.

        Args:
            resource: The resource to lock.
            ttl_ms: Time-to-live in milliseconds.
            timeout_ms: Maximum wait time in milliseconds.

        Returns:
            A LockGuard on success.

        Raises:
            LockTimeoutError: If the timeout is exceeded.
        """

    @abstractmethod
    async def extend(self, guard: LockGuard, ttl_ms: int) -> LockGuard:
        """Extend the TTL of a held lock.

        Args:
            guard: The current lock guard.
            ttl_ms: New TTL in milliseconds from now.

        Returns:
            An updated LockGuard with the new expiry.
        """

    @abstractmethod
    async def unlock(self, guard: LockGuard) -> None:
        """Release a held lock.

        Args:
            guard: The lock guard to release.
        """


class DbDistributedLock(DistributedLock):
    """PostgreSQL-backed distributed lock using INSERT ON CONFLICT.

    Args:
        pool: An asyncpg connection pool.
        node_id: Unique identifier for this node.
        config: Lock configuration.
        metrics: Optional metrics collector.
    """

    _SQL_CREATE_TABLE = """
        CREATE TABLE IF NOT EXISTS {table} (
            resource    TEXT PRIMARY KEY,
            owner_id    TEXT NOT NULL,
            fence_token BIGINT NOT NULL DEFAULT 1,
            expires_at  DOUBLE PRECISION NOT NULL
        )
    """

    _SQL_TRY_LOCK = """
        INSERT INTO {table} (resource, owner_id, fence_token, expires_at)
        VALUES ($1, $2, 1, $3)
        ON CONFLICT (resource) DO UPDATE
            SET owner_id    = EXCLUDED.owner_id,
                fence_token = {table}.fence_token + 1,
                expires_at  = EXCLUDED.expires_at
            WHERE {table}.expires_at < $4
        RETURNING fence_token, expires_at
    """

    _SQL_EXTEND = """
        UPDATE {table}
        SET expires_at = $1
        WHERE resource = $2 AND owner_id = $3 AND fence_token = $4
        RETURNING fence_token, expires_at
    """

    _SQL_UNLOCK = """
        DELETE FROM {table}
        WHERE resource = $1 AND owner_id = $2 AND fence_token = $3
    """

    def __init__(
        self,
        pool: asyncpg.Pool,  # type: ignore[type-arg]
        node_id: str | None = None,
        config: LockConfig | None = None,
        metrics: LockMetrics | None = None,
    ) -> None:
        self._pool = pool
        self._node_id = node_id or uuid.uuid4().hex
        self._config = config or LockConfig()
        self._metrics = metrics
        self._table = self._config.table_name

    async def ensure_table(self) -> None:
        """Create the distributed lock table if it does not exist."""
        async with self._pool.acquire() as conn:
            await conn.execute(self._SQL_CREATE_TABLE.format(table=self._table))

    async def try_lock(self, resource: str, ttl_ms: int | None = None) -> LockGuard | None:
        """Attempt to acquire a lock without waiting.

        Args:
            resource: The resource to lock.
            ttl_ms: Time-to-live in milliseconds.

        Returns:
            A LockGuard if acquired, None otherwise.
        """
        ttl = ttl_ms or self._config.default_ttl_ms
        now = time.time()
        expires = now + ttl / 1000.0

        async with self._pool.acquire() as conn:
            row = await conn.fetchrow(
                self._SQL_TRY_LOCK.format(table=self._table),
                resource,
                self._node_id,
                expires,
                now,
            )

        if row is None:
            if self._metrics:
                self._metrics.acquisitions_total.labels(result="failed").inc()
            return None

        guard = LockGuard(
            resource=resource,
            owner_id=self._node_id,
            fence_token=row["fence_token"],
            expires_at=row["expires_at"],
            _lock_impl=self,
        )

        if self._metrics:
            self._metrics.acquisitions_total.labels(result="success").inc()
            self._metrics.held_count.inc()

        logger.debug("Acquired lock on %s (token=%d)", resource, guard.fence_token)
        return guard

    async def lock(self, resource: str, ttl_ms: int | None = None, timeout_ms: int | None = None) -> LockGuard:
        """Acquire a lock with retries.

        Args:
            resource: The resource to lock.
            ttl_ms: Time-to-live in milliseconds.
            timeout_ms: Maximum wait time in milliseconds.

        Returns:
            A LockGuard on success.

        Raises:
            LockTimeoutError: If timeout is exceeded.
        """
        timeout = timeout_ms or (self._config.retry_delay_ms * self._config.max_retries)
        deadline = time.time() + timeout / 1000.0
        start = time.time()

        for _ in range(self._config.max_retries):
            guard = await self.try_lock(resource, ttl_ms)
            if guard is not None:
                if self._metrics:
                    self._metrics.wait_duration_seconds.observe(time.time() - start)
                return guard

            if time.time() >= deadline:
                break

            await asyncio.sleep(self._config.retry_delay_ms / 1000.0)

        msg = f"Timed out acquiring lock on {resource} after {timeout}ms"
        raise LockTimeoutError(msg)

    async def extend(self, guard: LockGuard, ttl_ms: int) -> LockGuard:
        """Extend the TTL of a held lock.

        Args:
            guard: The current lock guard.
            ttl_ms: New TTL in milliseconds from now.

        Returns:
            An updated LockGuard.
        """
        new_expires = time.time() + ttl_ms / 1000.0

        async with self._pool.acquire() as conn:
            row = await conn.fetchrow(
                self._SQL_EXTEND.format(table=self._table),
                new_expires,
                guard.resource,
                guard.owner_id,
                guard.fence_token,
            )

        if row is None:
            msg = f"Cannot extend lock on {guard.resource}: lock not held"
            raise LockTimeoutError(msg)

        return LockGuard(
            resource=guard.resource,
            owner_id=guard.owner_id,
            fence_token=row["fence_token"],
            expires_at=row["expires_at"],
            _lock_impl=self,
        )

    async def unlock(self, guard: LockGuard) -> None:
        """Release a held lock.

        Args:
            guard: The lock guard to release.
        """
        async with self._pool.acquire() as conn:
            await conn.execute(
                self._SQL_UNLOCK.format(table=self._table),
                guard.resource,
                guard.owner_id,
                guard.fence_token,
            )

        if self._metrics:
            self._metrics.releases_total.inc()
            self._metrics.held_count.dec()

        logger.debug("Released lock on %s", guard.resource)


class RedisDistributedLock(DistributedLock):
    """Redis-backed distributed lock using Lua scripts for atomicity.

    Args:
        client: An async Redis client.
        node_id: Unique identifier for this node.
        config: Lock configuration.
        metrics: Optional metrics collector.
    """

    _LUA_LOCK = """
        if redis.call("exists", KEYS[1]) == 0 then
            redis.call("hset", KEYS[1], "owner", ARGV[1], "token", ARGV[2])
            redis.call("pexpire", KEYS[1], ARGV[3])
            return ARGV[2]
        end
        return nil
    """

    _LUA_EXTEND = """
        if redis.call("hget", KEYS[1], "owner") == ARGV[1]
           and redis.call("hget", KEYS[1], "token") == ARGV[2] then
            redis.call("pexpire", KEYS[1], ARGV[3])
            return 1
        end
        return 0
    """

    _LUA_UNLOCK = """
        if redis.call("hget", KEYS[1], "owner") == ARGV[1]
           and redis.call("hget", KEYS[1], "token") == ARGV[2] then
            return redis.call("del", KEYS[1])
        end
        return 0
    """

    def __init__(
        self,
        client: aioredis.Redis,  # type: ignore[type-arg]
        node_id: str | None = None,
        config: LockConfig | None = None,
        metrics: LockMetrics | None = None,
    ) -> None:
        self._client = client
        self._node_id = node_id or uuid.uuid4().hex
        self._config = config or LockConfig()
        self._metrics = metrics
        self._token_counter = 0

    def _next_token(self) -> int:
        self._token_counter += 1
        return self._token_counter

    async def try_lock(self, resource: str, ttl_ms: int | None = None) -> LockGuard | None:
        """Attempt to acquire a Redis lock without waiting.

        Args:
            resource: The resource to lock.
            ttl_ms: Time-to-live in milliseconds.

        Returns:
            A LockGuard if acquired, None otherwise.
        """
        ttl = ttl_ms or self._config.default_ttl_ms
        token = self._next_token()
        key = f"k1s0:lock:{resource}"

        result = await self._client.eval(
            self._LUA_LOCK,
            1,
            key,
            self._node_id,
            str(token),
            str(ttl),
        )

        if result is None:
            if self._metrics:
                self._metrics.acquisitions_total.labels(result="failed").inc()
            return None

        guard = LockGuard(
            resource=resource,
            owner_id=self._node_id,
            fence_token=token,
            expires_at=time.time() + ttl / 1000.0,
            _lock_impl=self,
        )

        if self._metrics:
            self._metrics.acquisitions_total.labels(result="success").inc()
            self._metrics.held_count.inc()

        return guard

    async def lock(self, resource: str, ttl_ms: int | None = None, timeout_ms: int | None = None) -> LockGuard:
        """Acquire a Redis lock with retries.

        Args:
            resource: The resource to lock.
            ttl_ms: Time-to-live in milliseconds.
            timeout_ms: Maximum wait time in milliseconds.

        Returns:
            A LockGuard on success.

        Raises:
            LockTimeoutError: If timeout is exceeded.
        """
        timeout = timeout_ms or (self._config.retry_delay_ms * self._config.max_retries)
        deadline = time.time() + timeout / 1000.0
        start = time.time()

        for _ in range(self._config.max_retries):
            guard = await self.try_lock(resource, ttl_ms)
            if guard is not None:
                if self._metrics:
                    self._metrics.wait_duration_seconds.observe(time.time() - start)
                return guard

            if time.time() >= deadline:
                break

            await asyncio.sleep(self._config.retry_delay_ms / 1000.0)

        msg = f"Timed out acquiring Redis lock on {resource} after {timeout}ms"
        raise LockTimeoutError(msg)

    async def extend(self, guard: LockGuard, ttl_ms: int) -> LockGuard:
        """Extend the TTL of a held Redis lock.

        Args:
            guard: The current lock guard.
            ttl_ms: New TTL in milliseconds from now.

        Returns:
            An updated LockGuard.
        """
        key = f"k1s0:lock:{guard.resource}"

        result = await self._client.eval(
            self._LUA_EXTEND,
            1,
            key,
            guard.owner_id,
            str(guard.fence_token),
            str(ttl_ms),
        )

        if result == 0:
            msg = f"Cannot extend Redis lock on {guard.resource}: lock not held"
            raise LockTimeoutError(msg)

        return LockGuard(
            resource=guard.resource,
            owner_id=guard.owner_id,
            fence_token=guard.fence_token,
            expires_at=time.time() + ttl_ms / 1000.0,
            _lock_impl=self,
        )

    async def unlock(self, guard: LockGuard) -> None:
        """Release a held Redis lock.

        Args:
            guard: The lock guard to release.
        """
        key = f"k1s0:lock:{guard.resource}"

        await self._client.eval(
            self._LUA_UNLOCK,
            1,
            key,
            guard.owner_id,
            str(guard.fence_token),
        )

        if self._metrics:
            self._metrics.releases_total.inc()
            self._metrics.held_count.dec()
