"""イベントプロデューサー"""

from __future__ import annotations

from abc import ABC, abstractmethod
from typing import Any

from .exceptions import MessagingError, MessagingErrorCodes
from .models import EventEnvelope


class EventProducer(ABC):
    """イベントプロデューサー抽象基底クラス。"""

    @abstractmethod
    def publish(self, envelope: EventEnvelope) -> None:
        """イベントを発行する（同期）。"""
        ...

    @abstractmethod
    async def publish_async(self, envelope: EventEnvelope) -> None:
        """イベントを発行する（非同期）。"""
        ...

    @abstractmethod
    def publish_batch(self, envelopes: list[EventEnvelope]) -> None:
        """複数イベントをバッチ発行する。"""
        ...

    @abstractmethod
    def close(self) -> None:
        """プロデューサーを閉じる。"""
        ...

    def __enter__(self) -> EventProducer:
        return self

    def __exit__(self, *args: object) -> None:
        self.close()


class KafkaEventProducer(EventProducer):
    """confluent-kafka を使ったイベントプロデューサー。"""

    def __init__(self, brokers: list[str], timeout_seconds: float = 10.0) -> None:
        self._brokers = brokers
        self._timeout_seconds = timeout_seconds
        self._producer: Any = None

    def _get_producer(self) -> Any:  # noqa: ANN401
        if self._producer is None:
            from confluent_kafka import Producer

            self._producer = Producer({"bootstrap.servers": ",".join(self._brokers)})
        return self._producer

    def publish(self, envelope: EventEnvelope) -> None:
        """Kafka にイベントを発行する。"""
        try:
            producer = self._get_producer()
            headers = [(k, v.encode()) for k, v in envelope.headers.items()]
            producer.produce(
                topic=envelope.topic,
                value=envelope.payload,
                key=envelope.key,
                headers=headers,
            )
            producer.flush(timeout=self._timeout_seconds)
        except Exception as e:
            raise MessagingError(
                code=MessagingErrorCodes.PUBLISH_FAILED,
                message=f"Failed to publish event to {envelope.topic}: {e}",
                cause=e,
            ) from e

    async def publish_async(self, envelope: EventEnvelope) -> None:
        """非同期でイベントを発行する（実装は同期を実行）。"""
        import asyncio

        loop = asyncio.get_event_loop()
        await loop.run_in_executor(None, self.publish, envelope)

    def publish_batch(self, envelopes: list[EventEnvelope]) -> None:
        """複数イベントをバッチで発行する。"""
        try:
            producer = self._get_producer()
            for envelope in envelopes:
                headers = [(k, v.encode()) for k, v in envelope.headers.items()]
                producer.produce(
                    topic=envelope.topic,
                    value=envelope.payload,
                    key=envelope.key,
                    headers=headers,
                )
            producer.flush(timeout=self._timeout_seconds)
        except Exception as e:
            raise MessagingError(
                code=MessagingErrorCodes.PUBLISH_FAILED,
                message=f"Failed to publish batch: {e}",
                cause=e,
            ) from e

    def close(self) -> None:
        """プロデューサーをフラッシュして閉じる。"""
        if self._producer is not None:
            self._producer.flush()
            self._producer = None
