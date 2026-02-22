"""DLQ クライアントデータモデル"""

from __future__ import annotations

import uuid
from dataclasses import dataclass
from enum import StrEnum
from typing import Any


class DlqStatus(StrEnum):
    """DLQ メッセージステータス。"""

    PENDING = "PENDING"
    RETRYING = "RETRYING"
    RESOLVED = "RESOLVED"
    DEAD = "DEAD"


@dataclass
class DlqMessage:
    """DLQ メッセージ。"""

    id: uuid.UUID
    original_topic: str
    error_message: str
    retry_count: int
    max_retries: int
    payload: bytes
    status: DlqStatus
    created_at: str = ""
    updated_at: str = ""

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> DlqMessage:
        """API レスポンス辞書から DlqMessage を生成する。"""
        return cls(
            id=uuid.UUID(data["id"]),
            original_topic=data["original_topic"],
            error_message=data.get("error_message", ""),
            retry_count=data.get("retry_count", 0),
            max_retries=data.get("max_retries", 3),
            payload=(raw.encode() if isinstance(raw := data.get("payload", b""), str) else raw),
            status=DlqStatus(data.get("status", "PENDING")),
            created_at=data.get("created_at", ""),
            updated_at=data.get("updated_at", ""),
        )


@dataclass
class ListDlqMessagesResponse:
    """DLQ メッセージ一覧レスポンス。"""

    messages: list[DlqMessage]
    total: int
    page: int
    page_size: int

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> ListDlqMessagesResponse:
        return cls(
            messages=[DlqMessage.from_dict(m) for m in data.get("messages", [])],
            total=data.get("total", 0),
            page=data.get("page", 1),
            page_size=data.get("page_size", 20),
        )


@dataclass
class RetryDlqMessageResponse:
    """DLQ メッセージリトライレスポンス。"""

    message_id: uuid.UUID
    status: DlqStatus
    message: str = ""

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> RetryDlqMessageResponse:
        return cls(
            message_id=uuid.UUID(data["message_id"]),
            status=DlqStatus(data.get("status", "RETRYING")),
            message=data.get("message", ""),
        )


@dataclass
class DlqConfig:
    """DLQ クライアント設定。"""

    base_url: str
    api_key: str = ""
    timeout_seconds: float = 10.0
