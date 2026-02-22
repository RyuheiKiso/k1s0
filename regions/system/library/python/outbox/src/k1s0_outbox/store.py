"""OutboxStore 抽象基底クラス"""

from __future__ import annotations

import uuid
from abc import ABC, abstractmethod

from .models import OutboxMessage


class OutboxStore(ABC):
    """Outbox ストレージ抽象基底クラス。"""

    @abstractmethod
    async def save(self, message: OutboxMessage) -> None:
        """メッセージを保存する。"""
        ...

    @abstractmethod
    async def fetch_pending(self, limit: int = 100) -> list[OutboxMessage]:
        """PENDING 状態のメッセージを取得する。"""
        ...

    @abstractmethod
    async def mark_published(self, message_id: uuid.UUID) -> None:
        """メッセージを PUBLISHED に更新する。"""
        ...

    @abstractmethod
    async def mark_failed(self, message_id: uuid.UUID, error: str) -> None:
        """メッセージを FAILED に更新する。"""
        ...

    @abstractmethod
    async def increment_retry(self, message_id: uuid.UUID) -> None:
        """リトライカウントをインクリメントする。"""
        ...
