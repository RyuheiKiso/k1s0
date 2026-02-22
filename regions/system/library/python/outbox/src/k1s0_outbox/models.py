"""Outbox データモデル"""

from __future__ import annotations

import uuid
from dataclasses import dataclass, field
from datetime import UTC, datetime
from enum import StrEnum


class OutboxStatus(StrEnum):
    """Outbox メッセージステータス。"""

    PENDING = "PENDING"
    PUBLISHED = "PUBLISHED"
    FAILED = "FAILED"


@dataclass
class RetryConfig:
    """リトライ設定。"""

    max_retries: int = 3
    base_delay_seconds: float = 1.0
    max_delay_seconds: float = 60.0
    multiplier: float = 2.0


@dataclass
class OutboxConfig:
    """Outbox 設定。"""

    polling_interval_seconds: float = 1.0
    batch_size: int = 100
    retry: RetryConfig = field(default_factory=RetryConfig)


@dataclass
class OutboxMessage:
    """Outbox メッセージ。"""

    id: uuid.UUID = field(default_factory=uuid.uuid4)
    topic: str = ""
    payload: bytes = b""
    status: OutboxStatus = OutboxStatus.PENDING
    retry_count: int = 0
    created_at: datetime = field(default_factory=lambda: datetime.now(UTC))
    updated_at: datetime = field(default_factory=lambda: datetime.now(UTC))
    error_message: str | None = None
    headers: dict[str, str] = field(default_factory=dict)

    def __post_init__(self) -> None:
        if not self.topic:
            raise ValueError("topic cannot be empty")
