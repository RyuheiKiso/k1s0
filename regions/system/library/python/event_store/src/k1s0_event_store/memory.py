"""InMemoryEventStore 実装"""

from __future__ import annotations

from .exceptions import VersionConflictError
from .models import EventEnvelope
from .store import EventStore


class InMemoryEventStore(EventStore):
    """テスト用インメモリイベントストア。"""

    def __init__(self) -> None:
        self._streams: dict[str, list[EventEnvelope]] = {}

    async def append(
        self,
        stream_id: str,
        events: list[EventEnvelope],
        expected_version: int | None = None,
    ) -> int:
        stream = self._streams.get(stream_id, [])
        cur_version = len(stream)

        if expected_version is not None and expected_version != cur_version:
            raise VersionConflictError(expected=expected_version, actual=cur_version)

        for i, event in enumerate(events):
            event.stream_id = stream_id
            event.version = cur_version + i + 1
            stream.append(event)

        self._streams[stream_id] = stream
        return len(stream)

    async def load(self, stream_id: str) -> list[EventEnvelope]:
        return list(self._streams.get(stream_id, []))

    async def load_from(self, stream_id: str, from_version: int) -> list[EventEnvelope]:
        stream = self._streams.get(stream_id, [])
        return [e for e in stream if e.version > from_version]

    async def exists(self, stream_id: str) -> bool:
        return stream_id in self._streams and len(self._streams[stream_id]) > 0

    async def current_version(self, stream_id: str) -> int:
        return len(self._streams.get(stream_id, []))
