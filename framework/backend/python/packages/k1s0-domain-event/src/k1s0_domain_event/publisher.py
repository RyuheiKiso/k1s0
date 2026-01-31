"""Event publisher abstract base class."""

from __future__ import annotations

from abc import ABC, abstractmethod

from k1s0_domain_event.envelope import EventEnvelope


class EventPublisher(ABC):
    """Abstract base class for publishing domain event envelopes."""

    @abstractmethod
    async def publish(self, envelope: EventEnvelope) -> None:
        """Publish a single event envelope.

        Args:
            envelope: The event envelope to publish.

        Raises:
            PublishError: If the event could not be published.
        """

    @abstractmethod
    async def publish_batch(self, envelopes: list[EventEnvelope]) -> None:
        """Publish multiple event envelopes.

        Args:
            envelopes: The event envelopes to publish.

        Raises:
            PublishError: If any event could not be published.
        """
