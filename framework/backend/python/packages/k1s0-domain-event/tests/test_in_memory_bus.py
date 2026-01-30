"""Tests for InMemoryEventBus."""

from __future__ import annotations

import pytest

from k1s0_domain_event.bus.in_memory import InMemoryEventBus
from k1s0_domain_event.envelope import EventEnvelope, EventMetadata
from k1s0_domain_event.subscriber import EventHandler


def _make_envelope(event_type: str = "test.event") -> EventEnvelope:
    return EventEnvelope(
        event_type=event_type,
        metadata=EventMetadata(source="test"),
        payload="{}",
    )


class _RecordingHandler(EventHandler):
    def __init__(self, et: str = "test.event") -> None:
        self._et = et
        self.received: list[EventEnvelope] = []

    def event_type(self) -> str:
        return self._et

    async def handle(self, envelope: EventEnvelope) -> None:
        self.received.append(envelope)


@pytest.mark.asyncio
class TestInMemoryEventBus:
    async def test_publish_dispatches_to_handler(self) -> None:
        bus = InMemoryEventBus()
        handler = _RecordingHandler()
        await bus.subscribe(handler)

        envelope = _make_envelope()
        await bus.publish(envelope)

        assert len(handler.received) == 1
        assert handler.received[0] is envelope

    async def test_publish_multiple_handlers(self) -> None:
        bus = InMemoryEventBus()
        h1 = _RecordingHandler()
        h2 = _RecordingHandler()
        await bus.subscribe(h1)
        await bus.subscribe(h2)

        await bus.publish(_make_envelope())

        assert len(h1.received) == 1
        assert len(h2.received) == 1

    async def test_publish_ignores_non_matching_handlers(self) -> None:
        bus = InMemoryEventBus()
        handler = _RecordingHandler("other.event")
        await bus.subscribe(handler)

        await bus.publish(_make_envelope("test.event"))

        assert len(handler.received) == 0

    async def test_subscribe_returns_cancel(self) -> None:
        bus = InMemoryEventBus()
        handler = _RecordingHandler()
        cancel = await bus.subscribe(handler)

        await cancel()
        await bus.publish(_make_envelope())

        assert len(handler.received) == 0

    async def test_publish_batch_delivers_all(self) -> None:
        bus = InMemoryEventBus()
        handler = _RecordingHandler()
        await bus.subscribe(handler)

        envelopes = [_make_envelope() for _ in range(3)]
        await bus.publish_batch(envelopes)

        assert len(handler.received) == 3
