"""Audit client for recording audit events."""

from __future__ import annotations

import uuid
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from datetime import datetime, timezone


@dataclass
class AuditEvent:
    """Audit event record."""

    tenant_id: str
    actor_id: str
    action: str
    resource_type: str
    resource_id: str
    id: str = field(default_factory=lambda: str(uuid.uuid4()))
    timestamp: datetime = field(
        default_factory=lambda: datetime.now(timezone.utc)
    )


class AuditClient(ABC):
    """Abstract audit client."""

    @abstractmethod
    async def record(self, event: AuditEvent) -> None: ...

    @abstractmethod
    async def flush(self) -> list[AuditEvent]: ...


class BufferedAuditClient(AuditClient):
    """Buffered audit client that stores events in memory."""

    def __init__(self) -> None:
        self._buffer: list[AuditEvent] = []

    async def record(self, event: AuditEvent) -> None:
        self._buffer.append(event)

    async def flush(self) -> list[AuditEvent]:
        result = list(self._buffer)
        self._buffer.clear()
        return result
