"""In-memory event bus implementing both publisher and subscriber."""

from __future__ import annotations

from collections.abc import Callable, Coroutine
from typing import Any

from k1s0_domain_event.envelope import EventEnvelope
from k1s0_domain_event.errors import PublishError
from k1s0_domain_event.publisher import EventPublisher
from k1s0_domain_event.subscriber import EventHandler, EventSubscriber


class InMemoryEventBus(EventPublisher, EventSubscriber):
    """In-memory event bus for testing and single-process use cases.

    Dispatches events synchronously to all registered handlers
    matching the event type.
    """

    def __init__(self) -> None:
        self._handlers: dict[str, list[EventHandler]] = {}

    async def publish(self, envelope: EventEnvelope) -> None:
        """Publish an event to all matching handlers.

        Args:
            envelope: The event envelope to publish.

        Raises:
            PublishError: If any handler raises an exception.
        """
        handlers = self._handlers.get(envelope.event_type, [])
        for handler in handlers:
            try:
                await handler.handle(envelope)
            except Exception as exc:
                raise PublishError(
                    f"Handler failed for event type '{envelope.event_type}': {exc}"
                ) from exc

    async def publish_batch(self, envelopes: list[EventEnvelope]) -> None:
        """Publish multiple events sequentially.

        Args:
            envelopes: The event envelopes to publish.

        Raises:
            PublishError: If any handler raises an exception.
        """
        for envelope in envelopes:
            await self.publish(envelope)

    async def subscribe(
        self, handler: EventHandler
    ) -> Callable[[], Coroutine[Any, Any, None]]:
        """Subscribe a handler for its declared event type.

        Args:
            handler: The event handler to subscribe.

        Returns:
            An async callable that cancels the subscription.
        """
        event_type = handler.event_type()
        if event_type not in self._handlers:
            self._handlers[event_type] = []
        self._handlers[event_type].append(handler)

        async def cancel() -> None:
            handlers = self._handlers.get(event_type, [])
            if handler in handlers:
                handlers.remove(handler)

        return cancel
