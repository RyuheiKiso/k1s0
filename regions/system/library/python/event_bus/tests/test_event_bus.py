"""event_bus library unit tests."""

from k1s0_event_bus import Event, InMemoryEventBus


async def test_publish_to_subscriber() -> None:
    bus = InMemoryEventBus()
    received: list[Event] = []
    bus.subscribe("user.created", lambda e: _append(received, e))
    await bus.publish(Event(event_type="user.created", payload={"name": "Alice"}))
    assert len(received) == 1
    assert received[0].payload["name"] == "Alice"


async def test_no_subscriber() -> None:
    bus = InMemoryEventBus()
    # Should not raise
    await bus.publish(Event(event_type="unhandled", payload={}))


async def test_multiple_subscribers() -> None:
    bus = InMemoryEventBus()
    received1: list[Event] = []
    received2: list[Event] = []
    bus.subscribe("order.placed", lambda e: _append(received1, e))
    bus.subscribe("order.placed", lambda e: _append(received2, e))
    await bus.publish(Event(event_type="order.placed", payload={}))
    assert len(received1) == 1
    assert len(received2) == 1


async def test_unsubscribe() -> None:
    bus = InMemoryEventBus()
    received: list[Event] = []
    bus.subscribe("test.event", lambda e: _append(received, e))
    bus.unsubscribe("test.event")
    await bus.publish(Event(event_type="test.event", payload={}))
    assert len(received) == 0


async def test_event_has_id_and_timestamp() -> None:
    event = Event(event_type="test", payload={"key": "value"})
    assert event.id
    assert event.timestamp


async def test_different_event_types_isolated() -> None:
    bus = InMemoryEventBus()
    a_events: list[Event] = []
    b_events: list[Event] = []
    bus.subscribe("type_a", lambda e: _append(a_events, e))
    bus.subscribe("type_b", lambda e: _append(b_events, e))
    await bus.publish(Event(event_type="type_a", payload={}))
    assert len(a_events) == 1
    assert len(b_events) == 0


async def _append(lst: list[Event], event: Event) -> None:
    lst.append(event)
