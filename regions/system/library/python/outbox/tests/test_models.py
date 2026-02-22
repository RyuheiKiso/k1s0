"""Outbox モデルのユニットテスト"""

import uuid

import pytest
from k1s0_outbox.models import OutboxMessage, OutboxStatus


def test_outbox_message_creates_with_defaults() -> None:
    """OutboxMessage がデフォルト値で作成できること。"""
    msg = OutboxMessage(topic="events", payload=b"data")
    assert msg.status == OutboxStatus.PENDING
    assert msg.retry_count == 0
    assert isinstance(msg.id, uuid.UUID)


def test_outbox_message_empty_topic_raises() -> None:
    """空のトピックで ValueError が発生すること。"""
    with pytest.raises(ValueError, match="topic cannot be empty"):
        OutboxMessage(topic="", payload=b"data")


def test_outbox_status_values() -> None:
    """OutboxStatus の値が正しいこと。"""
    assert OutboxStatus.PENDING.value == "PENDING"
    assert OutboxStatus.PUBLISHED.value == "PUBLISHED"
    assert OutboxStatus.FAILED.value == "FAILED"
