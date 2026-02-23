"""EventStore 抽象基底クラス"""

from __future__ import annotations

from abc import ABC, abstractmethod

from .models import EventEnvelope


class EventStore(ABC):
    """イベントストア抽象基底クラス。"""

    @abstractmethod
    async def append(
        self,
        stream_id: str,
        events: list[EventEnvelope],
        expected_version: int | None = None,
    ) -> int:
        """イベントをストリームに追記する。新しいバージョン番号を返す。"""
        ...

    @abstractmethod
    async def load(self, stream_id: str) -> list[EventEnvelope]:
        """ストリームの全イベントを読み取る。"""
        ...

    @abstractmethod
    async def load_from(self, stream_id: str, from_version: int) -> list[EventEnvelope]:
        """指定バージョン以降のイベントを読み取る。"""
        ...

    @abstractmethod
    async def exists(self, stream_id: str) -> bool:
        """ストリームが存在するか確認する。"""
        ...

    @abstractmethod
    async def current_version(self, stream_id: str) -> int:
        """ストリームの現在のバージョンを返す。"""
        ...
