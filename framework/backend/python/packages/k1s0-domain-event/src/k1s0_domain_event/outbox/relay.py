"""Outbox relay for polling and publishing pending events."""

from __future__ import annotations

import asyncio
import json

from k1s0_domain_event.envelope import EventEnvelope, EventMetadata
from k1s0_domain_event.errors import OutboxError, PublishError
from k1s0_domain_event.outbox.entry import OutboxEntry
from k1s0_domain_event.outbox.store import OutboxStore
from k1s0_domain_event.publisher import EventPublisher


class OutboxRelay:
    """Polls the outbox store and publishes pending events.

    Runs a background polling loop that fetches pending outbox entries,
    publishes them via the configured publisher, and updates their status.
    """

    def __init__(
        self,
        store: OutboxStore,
        publisher: EventPublisher,
        poll_interval: float = 1.0,
        max_retries: int = 3,
    ) -> None:
        self._store = store
        self._publisher = publisher
        self._poll_interval = poll_interval
        self._max_retries = max_retries
        self._running = False

    async def start(self) -> None:
        """Start the polling loop.

        Runs until stop() is called. Each iteration fetches pending entries
        and attempts to publish them.
        """
        self._running = True
        while self._running:
            await self._process_batch()
            await asyncio.sleep(self._poll_interval)

    async def stop(self) -> None:
        """Signal the polling loop to stop gracefully."""
        self._running = False

    async def _process_batch(self) -> None:
        """Fetch pending entries and publish them."""
        try:
            entries = await self._store.fetch_pending()
        except OutboxError:
            return

        for entry in entries:
            await self._process_entry(entry)

    async def _process_entry(self, entry: OutboxEntry) -> None:
        """Attempt to publish a single outbox entry.

        Args:
            entry: The outbox entry to process.
        """
        if entry.retry_count >= self._max_retries:
            await self._store.mark_failed(entry.id)
            return

        try:
            payload_data = json.loads(entry.payload)
            metadata = EventMetadata(source="outbox-relay")
            envelope = EventEnvelope(
                event_type=entry.event_type,
                metadata=metadata,
                payload=json.dumps(payload_data),
            )
            await self._publisher.publish(envelope)
            await self._store.mark_published(entry.id)
        except (PublishError, Exception):
            entry.retry_count += 1
            await self._store.mark_failed(entry.id)
