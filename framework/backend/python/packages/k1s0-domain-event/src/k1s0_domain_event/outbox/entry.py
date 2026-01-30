"""Outbox entry model for the transactional outbox pattern."""

from __future__ import annotations

from datetime import UTC, datetime
from enum import Enum
from uuid import UUID, uuid4

from pydantic import BaseModel, Field


class OutboxStatus(str, Enum):
    """Status of an outbox entry."""

    PENDING = "pending"
    PUBLISHED = "published"
    FAILED = "failed"


class OutboxEntry(BaseModel):
    """Represents a single entry in the outbox table."""

    id: UUID = Field(default_factory=uuid4)
    event_type: str
    payload: str
    status: OutboxStatus = OutboxStatus.PENDING
    retry_count: int = 0
    created_at: datetime = Field(default_factory=lambda: datetime.now(UTC))
    updated_at: datetime = Field(default_factory=lambda: datetime.now(UTC))
