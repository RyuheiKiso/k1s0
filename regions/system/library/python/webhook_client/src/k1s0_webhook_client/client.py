"""Webhook client abstract class."""

from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from typing import Any


@dataclass
class WebhookPayload:
    """Webhook payload."""

    event_type: str
    timestamp: str
    data: dict[str, Any] = field(default_factory=dict)


class WebhookClient(ABC):
    """Abstract webhook client."""

    @abstractmethod
    async def send(self, url: str, payload: WebhookPayload) -> int: ...
