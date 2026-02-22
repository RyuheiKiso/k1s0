"""Kafka 設定モデル"""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import StrEnum


class SecurityProtocol(StrEnum):
    """Kafka セキュリティプロトコル。"""

    PLAINTEXT = "PLAINTEXT"
    SSL = "SSL"
    SASL_PLAINTEXT = "SASL_PLAINTEXT"
    SASL_SSL = "SASL_SSL"


@dataclass
class KafkaSaslConfig:
    """SASL 認証設定。"""

    mechanism: str = "PLAIN"
    username: str = ""
    password: str = ""


@dataclass
class KafkaConfig:
    """Kafka 接続設定。"""

    brokers: list[str]
    consumer_group: str = ""
    security_protocol: SecurityProtocol = SecurityProtocol.PLAINTEXT
    sasl: KafkaSaslConfig = field(default_factory=KafkaSaslConfig)
    session_timeout_ms: int = 30000
    request_timeout_ms: int = 30000
    topics: list[str] = field(default_factory=list)

    def to_confluent_config(self) -> dict[str, str]:
        """confluent-kafka 設定辞書に変換する。"""
        config: dict[str, str] = {
            "bootstrap.servers": ",".join(self.brokers),
        }
        if self.consumer_group:
            config["group.id"] = self.consumer_group
        if self.security_protocol != SecurityProtocol.PLAINTEXT:
            config["security.protocol"] = self.security_protocol.value
        if self.security_protocol in (SecurityProtocol.SASL_PLAINTEXT, SecurityProtocol.SASL_SSL):
            config["sasl.mechanism"] = self.sasl.mechanism
            config["sasl.username"] = self.sasl.username
            config["sasl.password"] = self.sasl.password
        return config


@dataclass
class TopicConfig:
    """Kafka トピック設定。"""

    name: str
    partitions: int = 1
    replication_factor: int = 1
    retention_ms: int = 7 * 24 * 60 * 60 * 1000  # 7 days
    cleanup_policy: str = "delete"

    def __post_init__(self) -> None:
        if not self.name:
            raise ValueError("topic name cannot be empty")
        if not validate_topic_name(self.name):
            raise ValueError(f"Invalid topic name: {self.name}")
        if self.partitions < 1:
            raise ValueError("partitions must be >= 1")
        if self.replication_factor < 1:
            raise ValueError("replication_factor must be >= 1")


def validate_topic_name(name: str) -> bool:
    """トピック名の命名規則を検証する。

    許可: 英数字、ハイフン、アンダースコア、ドット
    禁止: 空文字、特殊文字、256文字超
    """
    if not name or len(name) > 255:
        return False
    return all(c.isalnum() or c in ("-", "_", ".") for c in name)
