"""Abstract outbox store interface."""

from __future__ import annotations

from abc import ABC, abstractmethod
from uuid import UUID

from k1s0_domain_event.outbox.entry import OutboxEntry


class OutboxStore(ABC):
    """Abstract base class for outbox persistence."""

    @abstractmethod
    async def insert(self, entry: OutboxEntry) -> None:
        """Insert a new outbox entry.

        Args:
            entry: The outbox entry to insert.

        Raises:
            OutboxError: If the insert fails.
        """

    @abstractmethod
    async def fetch_pending(self, limit: int = 100) -> list[OutboxEntry]:
        """Fetch pending outbox entries ordered by creation time.

        Args:
            limit: Maximum number of entries to fetch.

        Returns:
            A list of pending outbox entries.

        Raises:
            OutboxError: If the fetch fails.
        """

    @abstractmethod
    async def mark_published(self, entry_id: UUID) -> None:
        """Mark an outbox entry as published.

        Args:
            entry_id: The ID of the entry to mark.

        Raises:
            OutboxError: If the update fails.
        """

    @abstractmethod
    async def mark_failed(self, entry_id: UUID) -> None:
        """Mark an outbox entry as failed.

        Args:
            entry_id: The ID of the entry to mark.

        Raises:
            OutboxError: If the update fails.
        """
