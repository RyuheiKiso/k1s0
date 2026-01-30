"""Domain event publish/subscribe and outbox pattern for k1s0 Python services."""

from __future__ import annotations

from k1s0_domain_event.bus.in_memory import InMemoryEventBus
from k1s0_domain_event.envelope import EventEnvelope, EventMetadata
from k1s0_domain_event.errors import (
    DomainEventError,
    OutboxError,
    PublishError,
    SubscribeError,
)
from k1s0_domain_event.event import DomainEvent
from k1s0_domain_event.outbox.entry import OutboxEntry, OutboxStatus
from k1s0_domain_event.outbox.relay import OutboxRelay
from k1s0_domain_event.outbox.store import OutboxStore
from k1s0_domain_event.publisher import EventPublisher
from k1s0_domain_event.subscriber import EventHandler, EventSubscriber

__all__ = [
    "DomainEvent",
    "DomainEventError",
    "EventEnvelope",
    "EventHandler",
    "EventMetadata",
    "EventPublisher",
    "EventSubscriber",
    "InMemoryEventBus",
    "OutboxEntry",
    "OutboxError",
    "OutboxRelay",
    "OutboxStatus",
    "OutboxStore",
    "PublishError",
    "SubscribeError",
]
