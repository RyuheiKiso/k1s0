"""In-memory event bus."""

from __future__ import annotations

import uuid
from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Any, Awaitable, Callable


@dataclass
class Event:
    """Event published on the bus."""

    event_type: str
    payload: dict[str, Any]
    id: str = field(default_factory=lambda: str(uuid.uuid4()))
    timestamp: datetime = field(
        default_factory=lambda: datetime.now(timezone.utc)
    )


EventHandler = Callable[[Event], Awaitable[None]]


class InMemoryEventBus:
    """In-memory publish/subscribe event bus."""

    def __init__(self) -> None:
        self._handlers: dict[str, list[EventHandler]] = {}

    def subscribe(self, event_type: str, handler: EventHandler) -> None:
        """Subscribe a handler to an event type."""
        self._handlers.setdefault(event_type, []).append(handler)

    def unsubscribe(self, event_type: str) -> None:
        """Remove all handlers for an event type."""
        self._handlers.pop(event_type, None)

    async def publish(self, event: Event) -> None:
        """Publish an event to all subscribed handlers."""
        for handler in self._handlers.get(event.event_type, []):
            await handler(event)
