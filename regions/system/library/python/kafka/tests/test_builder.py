"""KafkaConfigBuilder のユニットテスト"""

import pytest
from k1s0_kafka.builder import KafkaConfigBuilder
from k1s0_kafka.models import SecurityProtocol


def test_builder_minimal_config() -> None:
    """最小構成でビルドできること。"""
    config = KafkaConfigBuilder().brokers("localhost:9092").build()
    assert config.brokers == ["localhost:9092"]
    assert config.security_protocol == SecurityProtocol.PLAINTEXT


def test_builder_full_config() -> None:
    """フル構成でビルドできること。"""
    config = (
        KafkaConfigBuilder()
        .brokers("broker1:9092", "broker2:9092")
        .consumer_group("my-group")
        .security_protocol(SecurityProtocol.SASL_SSL)
        .sasl("PLAIN", "user", "password")
        .topics("events", "commands")
        .build()
    )
    assert len(config.brokers) == 2
    assert config.consumer_group == "my-group"
    assert config.security_protocol == SecurityProtocol.SASL_SSL
    assert config.topics == ["events", "commands"]


def test_builder_no_brokers_raises() -> None:
    """ブローカーなしでビルドすると ValueError が発生すること。"""
    with pytest.raises(ValueError, match="broker"):
        KafkaConfigBuilder().build()


def test_to_confluent_config() -> None:
    """confluent-kafka 設定辞書への変換確認。"""
    config = KafkaConfigBuilder().brokers("localhost:9092").consumer_group("grp").build()
    confluent = config.to_confluent_config()
    assert confluent["bootstrap.servers"] == "localhost:9092"
    assert confluent["group.id"] == "grp"


def test_to_confluent_config_sasl() -> None:
    """SASL 設定が含まれること。"""
    config = (
        KafkaConfigBuilder()
        .brokers("localhost:9092")
        .security_protocol(SecurityProtocol.SASL_PLAINTEXT)
        .sasl("PLAIN", "user", "pass")
        .build()
    )
    confluent = config.to_confluent_config()
    assert confluent["security.protocol"] == "SASL_PLAINTEXT"
    assert confluent["sasl.username"] == "user"
