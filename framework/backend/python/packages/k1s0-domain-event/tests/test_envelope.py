"""Tests for EventEnvelope wrapping."""

from __future__ import annotations

import json
from uuid import UUID

from k1s0_domain_event.envelope import EventEnvelope
from k1s0_domain_event.event import DomainEvent


class _UserCreated(DomainEvent):
    def __init__(self, user_id: str) -> None:
        self._user_id = user_id

    def event_type(self) -> str:
        return "user.created"

    def aggregate_id(self) -> str:
        return self._user_id

    def aggregate_type(self) -> str:
        return "User"


class TestEventEnvelopeWrap:
    def test_wrap_sets_event_type(self) -> None:
        event = _UserCreated("u-123")
        envelope = EventEnvelope.wrap(event, source="test-service")
        assert envelope.event_type == "user.created"

    def test_wrap_sets_source(self) -> None:
        event = _UserCreated("u-123")
        envelope = EventEnvelope.wrap(event, source="test-service")
        assert envelope.metadata.source == "test-service"

    def test_wrap_generates_uuid_event_id(self) -> None:
        event = _UserCreated("u-123")
        envelope = EventEnvelope.wrap(event, source="test-service")
        assert isinstance(envelope.metadata.event_id, UUID)

    def test_wrap_sets_occurred_at(self) -> None:
        event = _UserCreated("u-123")
        envelope = EventEnvelope.wrap(event, source="test-service")
        assert envelope.metadata.occurred_at is not None

    def test_wrap_serializes_payload(self) -> None:
        event = _UserCreated("u-123")
        envelope = EventEnvelope.wrap(event, source="test-service")
        payload = json.loads(envelope.payload)
        assert payload["aggregate_id"] == "u-123"
        assert payload["aggregate_type"] == "User"

    def test_wrap_with_correlation_id(self) -> None:
        event = _UserCreated("u-123")
        envelope = EventEnvelope.wrap(
            event, source="test-service", correlation_id="corr-1"
        )
        assert envelope.metadata.correlation_id == "corr-1"

    def test_wrap_without_optional_ids(self) -> None:
        event = _UserCreated("u-123")
        envelope = EventEnvelope.wrap(event, source="test-service")
        assert envelope.metadata.correlation_id is None
        assert envelope.metadata.causation_id is None
