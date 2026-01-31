"""Event handler and subscriber abstract base classes."""

from __future__ import annotations

from abc import ABC, abstractmethod
from collections.abc import Callable, Coroutine
from typing import Any

from k1s0_domain_event.envelope import EventEnvelope


class EventHandler(ABC):
    """Abstract base class for handling domain events of a specific type."""

    @abstractmethod
    def event_type(self) -> str:
        """Return the event type this handler processes."""

    @abstractmethod
    async def handle(self, envelope: EventEnvelope) -> None:
        """Handle a domain event envelope.

        Args:
            envelope: The event envelope to handle.
        """


class EventSubscriber(ABC):
    """Abstract base class for subscribing event handlers."""

    @abstractmethod
    async def subscribe(
        self, handler: EventHandler
    ) -> Callable[[], Coroutine[Any, Any, None]]:
        """Subscribe a handler for its declared event type.

        Args:
            handler: The event handler to subscribe.

        Returns:
            An async callable that, when awaited, cancels the subscription.

        Raises:
            SubscribeError: If the subscription could not be created.
        """
