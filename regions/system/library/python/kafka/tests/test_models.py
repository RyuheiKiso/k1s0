"""Kafka モデルのユニットテスト"""

import pytest
from k1s0_kafka.models import TopicConfig, validate_topic_name


def test_validate_topic_name_valid() -> None:
    """有効なトピック名のバリデーション。"""
    assert validate_topic_name("my-topic") is True
    assert validate_topic_name("my_topic") is True
    assert validate_topic_name("my.topic") is True
    assert validate_topic_name("topic123") is True


def test_validate_topic_name_invalid() -> None:
    """無効なトピック名のバリデーション。"""
    assert validate_topic_name("") is False
    assert validate_topic_name("topic with spaces") is False
    assert validate_topic_name("topic@special") is False
    assert validate_topic_name("a" * 256) is False


def test_topic_config_valid() -> None:
    """有効なトピック設定の作成。"""
    topic = TopicConfig(name="my-topic", partitions=3, replication_factor=2)
    assert topic.name == "my-topic"
    assert topic.partitions == 3


def test_topic_config_invalid_name() -> None:
    """無効なトピック名で ValueError が発生すること。"""
    with pytest.raises(ValueError, match="Invalid topic name"):
        TopicConfig(name="invalid topic!")


def test_topic_config_empty_name() -> None:
    """空のトピック名で ValueError が発生すること。"""
    with pytest.raises(ValueError):
        TopicConfig(name="")


def test_topic_config_invalid_partitions() -> None:
    """パーティション数が不正な場合に ValueError が発生すること。"""
    with pytest.raises(ValueError, match="partitions"):
        TopicConfig(name="valid-topic", partitions=0)
