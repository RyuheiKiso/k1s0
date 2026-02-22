"""インメモリ Outbox ストア（テスト用）"""

from __future__ import annotations

import uuid
from datetime import UTC, datetime

from .models import OutboxMessage, OutboxStatus
from .store import OutboxStore


class InMemoryOutboxStore(OutboxStore):
    """テスト用のインメモリ Outbox ストア。"""

    def __init__(self) -> None:
        self._messages: dict[uuid.UUID, OutboxMessage] = {}

    async def save(self, message: OutboxMessage) -> None:
        self._messages[message.id] = message

    async def fetch_pending(self, limit: int = 100) -> list[OutboxMessage]:
        return [m for m in list(self._messages.values()) if m.status == OutboxStatus.PENDING][
            :limit
        ]

    async def mark_published(self, message_id: uuid.UUID) -> None:
        if message_id in self._messages:
            msg = self._messages[message_id]
            msg.status = OutboxStatus.PUBLISHED
            msg.updated_at = datetime.now(UTC)

    async def mark_failed(self, message_id: uuid.UUID, error: str) -> None:
        if message_id in self._messages:
            msg = self._messages[message_id]
            msg.status = OutboxStatus.FAILED
            msg.error_message = error
            msg.updated_at = datetime.now(UTC)

    async def increment_retry(self, message_id: uuid.UUID) -> None:
        if message_id in self._messages:
            self._messages[message_id].retry_count += 1

    def all_messages(self) -> list[OutboxMessage]:
        return list(self._messages.values())
