"""Notification client for sending notifications."""

from __future__ import annotations

import uuid
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from enum import Enum


class NotificationChannel(str, Enum):
    """Notification channel types."""

    EMAIL = "email"
    SMS = "sms"
    PUSH = "push"
    WEBHOOK = "webhook"


@dataclass
class NotificationRequest:
    """Notification request."""

    channel: NotificationChannel
    recipient: str
    body: str
    id: str = field(default_factory=lambda: str(uuid.uuid4()))
    subject: str | None = None


@dataclass
class NotificationResponse:
    """Notification response."""

    id: str
    status: str
    message_id: str | None = None


class NotificationClient(ABC):
    """Abstract notification client."""

    @abstractmethod
    async def send(self, request: NotificationRequest) -> NotificationResponse: ...


class InMemoryNotificationClient(NotificationClient):
    """In-memory notification client for testing."""

    def __init__(self) -> None:
        self._sent: list[NotificationRequest] = []

    @property
    def sent(self) -> list[NotificationRequest]:
        """Get a copy of sent notifications."""
        return list(self._sent)

    async def send(self, request: NotificationRequest) -> NotificationResponse:
        self._sent.append(request)
        return NotificationResponse(id=request.id, status="sent")
