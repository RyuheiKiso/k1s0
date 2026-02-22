"""DlqClient 抽象基底クラス"""

from __future__ import annotations

import uuid
from abc import ABC, abstractmethod

from .models import DlqMessage, ListDlqMessagesResponse, RetryDlqMessageResponse


class DlqClient(ABC):
    """DLQ クライアント抽象基底クラス。"""

    @abstractmethod
    async def list_messages(
        self,
        topic: str,
        page: int = 1,
        page_size: int = 20,
    ) -> ListDlqMessagesResponse:
        """DLQ メッセージ一覧を取得する。"""
        ...

    @abstractmethod
    async def get_message(self, message_id: uuid.UUID) -> DlqMessage:
        """単一 DLQ メッセージを取得する。"""
        ...

    @abstractmethod
    async def retry_message(self, message_id: uuid.UUID) -> RetryDlqMessageResponse:
        """DLQ メッセージをリトライする。"""
        ...

    @abstractmethod
    async def delete_message(self, message_id: uuid.UUID) -> None:
        """DLQ メッセージを削除する。"""
        ...

    @abstractmethod
    async def retry_all(self, topic: str) -> None:
        """指定トピックの全 DLQ メッセージをリトライする。"""
        ...
