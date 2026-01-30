"""Event envelope and metadata for wrapping domain events."""

from __future__ import annotations

import json
from datetime import UTC, datetime
from uuid import UUID, uuid4

from pydantic import BaseModel, Field

from k1s0_domain_event.event import DomainEvent


class EventMetadata(BaseModel):
    """Metadata attached to every domain event envelope."""

    event_id: UUID = Field(default_factory=uuid4)
    occurred_at: datetime = Field(default_factory=lambda: datetime.now(UTC))
    source: str
    correlation_id: str | None = None
    causation_id: str | None = None


class EventEnvelope(BaseModel):
    """Wraps a domain event with metadata and a serialized payload."""

    event_type: str
    metadata: EventMetadata
    payload: str

    @classmethod
    def wrap(
        cls,
        event: DomainEvent,
        source: str,
        *,
        correlation_id: str | None = None,
        causation_id: str | None = None,
    ) -> EventEnvelope:
        """Create an envelope from a domain event.

        Args:
            event: The domain event to wrap.
            source: The source service or component name.
            correlation_id: Optional correlation ID for tracing.
            causation_id: Optional causation ID for tracing.

        Returns:
            A new EventEnvelope containing the serialized event.
        """
        payload = json.dumps(
            {
                "aggregate_id": event.aggregate_id(),
                "aggregate_type": event.aggregate_type(),
            }
        )
        metadata = EventMetadata(
            source=source,
            correlation_id=correlation_id,
            causation_id=causation_id,
        )
        return cls(
            event_type=event.event_type(),
            metadata=metadata,
            payload=payload,
        )
