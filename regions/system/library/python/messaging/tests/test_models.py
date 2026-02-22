"""messaging モデルのユニットテスト"""

import pytest
from k1s0_messaging.models import ConsumedMessage, EventEnvelope, EventMetadata


def test_event_envelope_creates_successfully() -> None:
    """EventEnvelope が正常に作成できること。"""
    envelope = EventEnvelope(topic="events", payload=b'{"key":"value"}')
    assert envelope.topic == "events"
    assert envelope.payload == b'{"key":"value"}'
    assert envelope.metadata.event_id  # 自動生成


def test_event_envelope_empty_topic_raises() -> None:
    """空のトピックで ValueError が発生すること。"""
    with pytest.raises(ValueError, match="topic cannot be empty"):
        EventEnvelope(topic="", payload=b"data")


def test_event_metadata_auto_id() -> None:
    """EventMetadata の event_id が自動生成されること。"""
    meta1 = EventMetadata()
    meta2 = EventMetadata()
    assert meta1.event_id != meta2.event_id


def test_event_metadata_timestamp_set() -> None:
    """timestamp が自動設定されること。"""
    meta = EventMetadata()
    assert meta.timestamp  # ISO 8601 形式の文字列


def test_consumed_message_creates() -> None:
    """ConsumedMessage が正常に作成できること。"""
    msg = ConsumedMessage(topic="events", partition=0, offset=100, payload=b"data")
    assert msg.topic == "events"
    assert msg.offset == 100
