"""messaging データモデル"""

from __future__ import annotations

import uuid
from dataclasses import dataclass, field
from datetime import UTC, datetime


@dataclass
class EventMetadata:
    """イベントメタデータ。"""

    event_id: str = field(default_factory=lambda: str(uuid.uuid4()))
    event_type: str = ""
    source: str = ""
    timestamp: str = field(default_factory=lambda: datetime.now(UTC).isoformat())
    trace_id: str = ""
    correlation_id: str = ""
    schema_version: str = "1.0"


@dataclass
class EventEnvelope:
    """イベントエンベロープ。"""

    topic: str
    payload: bytes
    metadata: EventMetadata = field(default_factory=EventMetadata)
    key: bytes | None = None
    headers: dict[str, str] = field(default_factory=dict)

    def __post_init__(self) -> None:
        if not self.topic:
            raise ValueError("topic cannot be empty")


@dataclass
class ConsumedMessage:
    """コンシューマーで受信したメッセージ。"""

    topic: str
    partition: int
    offset: int
    payload: bytes
    key: bytes | None = None
    headers: dict[str, str] = field(default_factory=dict)


@dataclass
class ConsumerConfig:
    """コンシューマー設定。"""

    brokers: list[str]
    group_id: str
    auto_offset_reset: str = "earliest"
    enable_auto_commit: bool = False
    max_poll_records: int = 500


@dataclass
class MessagingConfig:
    """メッセージング全体設定。"""

    brokers: list[str]
    producer_timeout_seconds: float = 10.0
    consumer_poll_timeout_seconds: float = 1.0
