"""Tests for OutboxRelay."""

from __future__ import annotations

from unittest.mock import AsyncMock, MagicMock
from uuid import uuid4

import pytest

from k1s0_domain_event.envelope import EventEnvelope
from k1s0_domain_event.errors import PublishError
from k1s0_domain_event.outbox.entry import OutboxEntry, OutboxStatus
from k1s0_domain_event.outbox.relay import OutboxRelay
from k1s0_domain_event.outbox.store import OutboxStore
from k1s0_domain_event.publisher import EventPublisher


def _make_entry(
    event_type: str = "test.event",
    retry_count: int = 0,
) -> OutboxEntry:
    return OutboxEntry(
        event_type=event_type,
        payload='{"key": "value"}',
        status=OutboxStatus.PENDING,
        retry_count=retry_count,
    )


def _mock_store(entries: list[OutboxEntry] | None = None) -> AsyncMock:
    store = AsyncMock(spec=OutboxStore)
    store.fetch_pending = AsyncMock(return_value=entries or [])
    store.mark_published = AsyncMock()
    store.mark_failed = AsyncMock()
    return store


def _mock_publisher(fail: bool = False) -> AsyncMock:
    publisher = AsyncMock(spec=EventPublisher)
    if fail:
        publisher.publish = AsyncMock(
            side_effect=PublishError("publish failed")
        )
    return publisher


@pytest.mark.asyncio
class TestOutboxRelay:
    async def test_process_publishes_and_marks_published(self) -> None:
        entry = _make_entry()
        store = _mock_store([entry])
        publisher = _mock_publisher()
        relay = OutboxRelay(store, publisher)

        await relay._process_batch()

        publisher.publish.assert_called_once()
        envelope: EventEnvelope = publisher.publish.call_args[0][0]
        assert envelope.event_type == "test.event"
        store.mark_published.assert_called_once_with(entry.id)

    async def test_publish_failure_marks_failed(self) -> None:
        entry = _make_entry()
        store = _mock_store([entry])
        publisher = _mock_publisher(fail=True)
        relay = OutboxRelay(store, publisher)

        await relay._process_batch()

        store.mark_failed.assert_called_once_with(entry.id)
        store.mark_published.assert_not_called()

    async def test_max_retries_exceeded_marks_failed(self) -> None:
        entry = _make_entry(retry_count=3)
        store = _mock_store([entry])
        publisher = _mock_publisher()
        relay = OutboxRelay(store, publisher, max_retries=3)

        await relay._process_batch()

        publisher.publish.assert_not_called()
        store.mark_failed.assert_called_once_with(entry.id)

    async def test_stop_ends_polling(self) -> None:
        store = _mock_store()
        publisher = _mock_publisher()
        relay = OutboxRelay(store, publisher, poll_interval=0.01)

        await relay.stop()
        assert relay._running is False
