"""イベントコンシューマー"""

from __future__ import annotations

from abc import ABC, abstractmethod
from typing import Any

from .exceptions import MessagingError, MessagingErrorCodes
from .models import ConsumedMessage, ConsumerConfig


class EventConsumer(ABC):
    """イベントコンシューマー抽象基底クラス。"""

    @abstractmethod
    def receive(self, timeout_seconds: float = 1.0) -> ConsumedMessage | None:
        """メッセージを受信する（同期）。タイムアウト時は None を返す。"""
        ...

    @abstractmethod
    async def receive_async(self, timeout_seconds: float = 1.0) -> ConsumedMessage | None:
        """メッセージを受信する（非同期）。"""
        ...

    @abstractmethod
    def commit(self, message: ConsumedMessage) -> None:
        """オフセットをコミットする。"""
        ...

    @abstractmethod
    def subscribe(self, topics: list[str]) -> None:
        """トピックを購読する。"""
        ...

    @abstractmethod
    def close(self) -> None:
        """コンシューマーを閉じる。"""
        ...


class KafkaEventConsumer(EventConsumer):
    """confluent-kafka を使ったイベントコンシューマー。"""

    def __init__(self, config: ConsumerConfig) -> None:
        self._config = config
        self._consumer: Any = None

    def _get_consumer(self) -> Any:  # noqa: ANN401
        if self._consumer is None:
            from confluent_kafka import Consumer

            self._consumer = Consumer(
                {
                    "bootstrap.servers": ",".join(self._config.brokers),
                    "group.id": self._config.group_id,
                    "auto.offset.reset": self._config.auto_offset_reset,
                    "enable.auto.commit": self._config.enable_auto_commit,
                }
            )
        return self._consumer

    def subscribe(self, topics: list[str]) -> None:
        """トピックを購読する。"""
        self._get_consumer().subscribe(topics)

    def receive(self, timeout_seconds: float = 1.0) -> ConsumedMessage | None:
        """Kafka からメッセージを受信する。"""
        try:
            consumer = self._get_consumer()
            msg = consumer.poll(timeout=timeout_seconds)
            if msg is None:
                return None
            if msg.error():
                raise MessagingError(
                    code=MessagingErrorCodes.RECEIVE_FAILED,
                    message=f"Kafka error: {msg.error()}",
                )
            headers: dict[str, str] = {}
            if msg.headers():
                headers = {k: v.decode() for k, v in msg.headers()}
            return ConsumedMessage(
                topic=msg.topic(),
                partition=msg.partition(),
                offset=msg.offset(),
                payload=msg.value() or b"",
                key=msg.key(),
                headers=headers,
            )
        except MessagingError:
            raise
        except Exception as e:
            raise MessagingError(
                code=MessagingErrorCodes.RECEIVE_FAILED,
                message=f"Failed to receive message: {e}",
                cause=e,
            ) from e

    async def receive_async(self, timeout_seconds: float = 1.0) -> ConsumedMessage | None:
        """非同期でメッセージを受信する。"""
        import asyncio

        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(None, self.receive, timeout_seconds)

    def commit(self, message: ConsumedMessage) -> None:
        """オフセットをコミットする（非同期）。"""
        from confluent_kafka import TopicPartition

        consumer = self._get_consumer()
        tp = TopicPartition(message.topic, message.partition, message.offset + 1)
        consumer.commit(offsets=[tp])

    def close(self) -> None:
        """コンシューマーを閉じる。"""
        if self._consumer is not None:
            self._consumer.close()
            self._consumer = None
