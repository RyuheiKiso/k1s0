"""テスト用 NoOp プロデューサー"""

from __future__ import annotations

from .models import EventEnvelope
from .producer import EventProducer


class NoOpEventProducer(EventProducer):
    """テスト用の NoOp イベントプロデューサー。発行したメッセージを記録する。"""

    def __init__(self) -> None:
        self.published: list[EventEnvelope] = []
        self.closed = False

    def publish(self, envelope: EventEnvelope) -> None:
        self.published.append(envelope)

    async def publish_async(self, envelope: EventEnvelope) -> None:
        self.published.append(envelope)

    def publish_batch(self, envelopes: list[EventEnvelope]) -> None:
        self.published.extend(envelopes)

    def close(self) -> None:
        self.closed = True
