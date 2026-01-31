"""Leader election with database-backed lease management."""

from __future__ import annotations

import asyncio
import logging
import time
import uuid
from abc import ABC, abstractmethod
from dataclasses import dataclass
from enum import Enum
from typing import AsyncIterator

import asyncpg

from k1s0_consensus.config import LeaderConfig
from k1s0_consensus.error import ConsensusError, LeaseExpiredError
from k1s0_consensus.metrics import LeaderMetrics

logger = logging.getLogger("k1s0.consensus.leader")


class LeaderEventType(Enum):
    """Types of leader election events."""

    ELECTED = "elected"
    LOST = "lost"
    RENEWED = "renewed"


@dataclass(frozen=True)
class LeaderLease:
    """Immutable representation of a leadership lease.

    Attributes:
        leader_id: Unique identifier of the leader node.
        resource: The resource being led (e.g., a service name).
        fence_token: Monotonically increasing token for fencing.
        acquired_at: Unix timestamp when the lease was acquired.
        expires_at: Unix timestamp when the lease expires.
    """

    leader_id: str
    resource: str
    fence_token: int
    acquired_at: float
    expires_at: float

    @property
    def is_expired(self) -> bool:
        """Check whether this lease has expired."""
        return time.time() > self.expires_at

    @property
    def remaining_ms(self) -> int:
        """Milliseconds remaining on this lease, clamped to zero."""
        return max(0, int((self.expires_at - time.time()) * 1000))


@dataclass(frozen=True)
class LeaderEvent:
    """An event emitted by the leader election subsystem.

    Attributes:
        event_type: The type of leader event.
        lease: The lease associated with this event, if any.
        timestamp: Unix timestamp when the event occurred.
    """

    event_type: LeaderEventType
    lease: LeaderLease | None
    timestamp: float


class LeaderElector(ABC):
    """Abstract interface for leader election.

    Implementations must provide database- or coordination-service-backed
    lease management with fencing tokens.
    """

    @abstractmethod
    async def try_acquire(self, resource: str) -> LeaderLease | None:
        """Attempt to acquire leadership for a resource.

        Args:
            resource: The resource identifier to lead.

        Returns:
            A LeaderLease if leadership was acquired, None otherwise.
        """

    @abstractmethod
    async def renew(self, lease: LeaderLease) -> LeaderLease:
        """Renew an existing leadership lease.

        Args:
            lease: The current lease to renew.

        Returns:
            A new LeaderLease with an updated expiry.

        Raises:
            LeaseExpiredError: If the lease has expired or been taken.
        """

    @abstractmethod
    async def release(self, lease: LeaderLease) -> None:
        """Voluntarily release leadership.

        Args:
            lease: The lease to release.
        """

    @abstractmethod
    async def current_leader(self, resource: str) -> LeaderLease | None:
        """Query who the current leader is for a resource.

        Args:
            resource: The resource identifier.

        Returns:
            The current LeaderLease, or None if no leader exists.
        """

    @abstractmethod
    async def watch(self, resource: str) -> AsyncIterator[LeaderEvent]:
        """Watch for leader changes on a resource.

        Args:
            resource: The resource identifier to watch.

        Yields:
            LeaderEvent instances as leadership changes occur.
        """


class DbLeaderElector(LeaderElector):
    """PostgreSQL-backed leader election using advisory-style row locks.

    Uses INSERT ON CONFLICT for atomic acquisition and UPDATE with
    fence token validation for renewals.

    Args:
        pool: An asyncpg connection pool.
        node_id: Unique identifier for this node.
        config: Leader election configuration.
        metrics: Optional metrics collector.
    """

    _SQL_CREATE_TABLE = """
        CREATE TABLE IF NOT EXISTS {table} (
            resource    TEXT PRIMARY KEY,
            leader_id   TEXT NOT NULL,
            fence_token BIGINT NOT NULL DEFAULT 1,
            acquired_at DOUBLE PRECISION NOT NULL,
            expires_at  DOUBLE PRECISION NOT NULL
        )
    """

    _SQL_TRY_ACQUIRE = """
        INSERT INTO {table} (resource, leader_id, fence_token, acquired_at, expires_at)
        VALUES ($1, $2, 1, $3, $4)
        ON CONFLICT (resource) DO UPDATE
            SET leader_id   = EXCLUDED.leader_id,
                fence_token = {table}.fence_token + 1,
                acquired_at = EXCLUDED.acquired_at,
                expires_at  = EXCLUDED.expires_at
            WHERE {table}.expires_at < $3
        RETURNING fence_token, acquired_at, expires_at
    """

    _SQL_RENEW = """
        UPDATE {table}
        SET expires_at = $1
        WHERE resource = $2
          AND leader_id = $3
          AND fence_token = $4
          AND expires_at >= $5
        RETURNING fence_token, acquired_at, expires_at
    """

    _SQL_RELEASE = """
        DELETE FROM {table}
        WHERE resource = $1 AND leader_id = $2 AND fence_token = $3
    """

    _SQL_CURRENT = """
        SELECT leader_id, fence_token, acquired_at, expires_at
        FROM {table}
        WHERE resource = $1 AND expires_at >= $2
    """

    def __init__(
        self,
        pool: asyncpg.Pool,  # type: ignore[type-arg]
        node_id: str | None = None,
        config: LeaderConfig | None = None,
        metrics: LeaderMetrics | None = None,
    ) -> None:
        self._pool = pool
        self._node_id = node_id or uuid.uuid4().hex
        self._config = config or LeaderConfig()
        self._metrics = metrics
        self._table = self._config.table_name
        self._heartbeat_task: asyncio.Task[None] | None = None
        self._current_lease: LeaderLease | None = None
        self._watchers: list[asyncio.Queue[LeaderEvent]] = []

    async def ensure_table(self) -> None:
        """Create the leader lease table if it does not exist."""
        async with self._pool.acquire() as conn:
            await conn.execute(self._SQL_CREATE_TABLE.format(table=self._table))

    async def try_acquire(self, resource: str) -> LeaderLease | None:
        """Attempt to acquire leadership for a resource.

        Args:
            resource: The resource identifier to lead.

        Returns:
            A LeaderLease if acquired, None otherwise.
        """
        now = time.time()
        expires = now + self._config.lease_duration_ms / 1000.0

        async with self._pool.acquire() as conn:
            row = await conn.fetchrow(
                self._SQL_TRY_ACQUIRE.format(table=self._table),
                resource,
                self._node_id,
                now,
                expires,
            )

        if row is None:
            if self._metrics:
                self._metrics.elections_total.labels(result="failed").inc()
            logger.debug("Failed to acquire leadership for %s", resource)
            return None

        lease = LeaderLease(
            leader_id=self._node_id,
            resource=resource,
            fence_token=row["fence_token"],
            acquired_at=row["acquired_at"],
            expires_at=row["expires_at"],
        )
        self._current_lease = lease

        if self._metrics:
            self._metrics.elections_total.labels(result="success").inc()
            self._metrics.is_leader.set(1)

        self._emit_event(LeaderEvent(LeaderEventType.ELECTED, lease, time.time()))
        logger.info("Acquired leadership for %s (token=%d)", resource, lease.fence_token)
        return lease

    async def renew(self, lease: LeaderLease) -> LeaderLease:
        """Renew an existing leadership lease.

        Args:
            lease: The current lease to renew.

        Returns:
            A new LeaderLease with updated expiry.

        Raises:
            LeaseExpiredError: If the lease cannot be renewed.
        """
        now = time.time()
        new_expires = now + self._config.lease_duration_ms / 1000.0

        async with self._pool.acquire() as conn:
            row = await conn.fetchrow(
                self._SQL_RENEW.format(table=self._table),
                new_expires,
                lease.resource,
                lease.leader_id,
                lease.fence_token,
                now,
            )

        if row is None:
            if self._metrics:
                self._metrics.renewals_total.labels(result="failed").inc()
                self._metrics.is_leader.set(0)
            self._current_lease = None
            self._emit_event(LeaderEvent(LeaderEventType.LOST, lease, time.time()))
            msg = f"Failed to renew lease for {lease.resource}: lease expired or taken"
            raise LeaseExpiredError(msg)

        renewed = LeaderLease(
            leader_id=lease.leader_id,
            resource=lease.resource,
            fence_token=row["fence_token"],
            acquired_at=row["acquired_at"],
            expires_at=row["expires_at"],
        )
        self._current_lease = renewed

        if self._metrics:
            self._metrics.renewals_total.labels(result="success").inc()

        self._emit_event(LeaderEvent(LeaderEventType.RENEWED, renewed, time.time()))
        return renewed

    async def release(self, lease: LeaderLease) -> None:
        """Voluntarily release leadership.

        Args:
            lease: The lease to release.
        """
        async with self._pool.acquire() as conn:
            await conn.execute(
                self._SQL_RELEASE.format(table=self._table),
                lease.resource,
                lease.leader_id,
                lease.fence_token,
            )

        if self._metrics:
            self._metrics.is_leader.set(0)

        self._current_lease = None
        self._emit_event(LeaderEvent(LeaderEventType.LOST, lease, time.time()))
        logger.info("Released leadership for %s", lease.resource)

    async def current_leader(self, resource: str) -> LeaderLease | None:
        """Query the current leader for a resource.

        Args:
            resource: The resource identifier.

        Returns:
            The current LeaderLease, or None.
        """
        now = time.time()
        async with self._pool.acquire() as conn:
            row = await conn.fetchrow(
                self._SQL_CURRENT.format(table=self._table),
                resource,
                now,
            )

        if row is None:
            return None

        return LeaderLease(
            leader_id=row["leader_id"],
            resource=resource,
            fence_token=row["fence_token"],
            acquired_at=row["acquired_at"],
            expires_at=row["expires_at"],
        )

    async def watch(self, resource: str) -> AsyncIterator[LeaderEvent]:
        """Watch for leader changes on a resource.

        Creates a queue that receives events whenever leadership changes.
        The caller should iterate this in an async for loop.

        Args:
            resource: The resource identifier to watch.

        Yields:
            LeaderEvent instances as they occur.
        """
        queue: asyncio.Queue[LeaderEvent] = asyncio.Queue()
        self._watchers.append(queue)
        try:
            while True:
                event = await queue.get()
                if event.lease is not None and event.lease.resource != resource:
                    continue
                yield event
        finally:
            self._watchers.remove(queue)

    def start_heartbeat(self, resource: str) -> None:
        """Start a background heartbeat task that renews the lease.

        The task runs at the configured renew_interval_ms and attempts to
        re-acquire if the lease is lost.

        Args:
            resource: The resource to maintain leadership for.
        """
        if self._heartbeat_task is not None:
            return

        async def _heartbeat() -> None:
            interval = self._config.renew_interval_ms / 1000.0
            while True:
                await asyncio.sleep(interval)
                lease = self._current_lease
                if lease is not None and lease.resource == resource:
                    try:
                        await self.renew(lease)
                    except LeaseExpiredError:
                        logger.warning("Heartbeat: lease lost for %s, attempting re-acquire", resource)
                        await self.try_acquire(resource)
                    except ConsensusError:
                        logger.exception("Heartbeat error for %s", resource)

        self._heartbeat_task = asyncio.create_task(_heartbeat())

    async def stop_heartbeat(self) -> None:
        """Stop the background heartbeat task."""
        if self._heartbeat_task is not None:
            self._heartbeat_task.cancel()
            try:
                await self._heartbeat_task
            except asyncio.CancelledError:
                pass
            self._heartbeat_task = None

    def _emit_event(self, event: LeaderEvent) -> None:
        """Push an event to all registered watcher queues."""
        for queue in self._watchers:
            queue.put_nowait(event)
