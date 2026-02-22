"""Kafka 設定ビルダー"""

from __future__ import annotations

from .models import KafkaConfig, KafkaSaslConfig, SecurityProtocol


class KafkaConfigBuilder:
    """KafkaConfig のビルダークラス。"""

    def __init__(self) -> None:
        self._brokers: list[str] = []
        self._consumer_group: str = ""
        self._security_protocol: SecurityProtocol = SecurityProtocol.PLAINTEXT
        self._sasl: KafkaSaslConfig = KafkaSaslConfig()
        self._session_timeout_ms: int = 30000
        self._request_timeout_ms: int = 30000
        self._topics: list[str] = []

    def brokers(self, *brokers: str) -> KafkaConfigBuilder:
        """ブローカーリストを設定する。"""
        self._brokers = list(brokers)
        return self

    def consumer_group(self, group: str) -> KafkaConfigBuilder:
        """コンシューマーグループを設定する。"""
        self._consumer_group = group
        return self

    def security_protocol(self, protocol: SecurityProtocol) -> KafkaConfigBuilder:
        """セキュリティプロトコルを設定する。"""
        self._security_protocol = protocol
        return self

    def sasl(self, mechanism: str, username: str, password: str) -> KafkaConfigBuilder:
        """SASL 認証を設定する。"""
        self._sasl = KafkaSaslConfig(mechanism=mechanism, username=username, password=password)
        return self

    def session_timeout_ms(self, timeout: int) -> KafkaConfigBuilder:
        """セッションタイムアウトを設定する。"""
        self._session_timeout_ms = timeout
        return self

    def topics(self, *topics: str) -> KafkaConfigBuilder:
        """トピックリストを設定する。"""
        self._topics = list(topics)
        return self

    def build(self) -> KafkaConfig:
        """KafkaConfig を生成する。"""
        if not self._brokers:
            raise ValueError("At least one broker must be specified")
        return KafkaConfig(
            brokers=self._brokers,
            consumer_group=self._consumer_group,
            security_protocol=self._security_protocol,
            sasl=self._sasl,
            session_timeout_ms=self._session_timeout_ms,
            request_timeout_ms=self._request_timeout_ms,
            topics=self._topics,
        )
