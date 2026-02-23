"""event_store ライブラリのユニットテスト"""

import pytest
from k1s0_event_store import (
    EventEnvelope,
    InMemoryEventStore,
    StreamNotFoundError,
    VersionConflictError,
)


def make_event(event_type: str = "TestEvent", payload: dict | None = None) -> EventEnvelope:
    return EventEnvelope(event_type=event_type, payload=payload or {})


async def test_append_and_load() -> None:
    """イベントの追記と読み取り。"""
    store = InMemoryEventStore()
    events = [make_event("Created"), make_event("Updated")]
    new_version = await store.append("stream-1", events, expected_version=0)
    assert new_version == 2

    loaded = await store.load("stream-1")
    assert len(loaded) == 2
    assert loaded[0].event_type == "Created"
    assert loaded[0].version == 1
    assert loaded[1].version == 2


async def test_load_empty_stream() -> None:
    """空ストリームの読み取り。"""
    store = InMemoryEventStore()
    loaded = await store.load("nonexistent")
    assert loaded == []


async def test_version_conflict() -> None:
    """楽観的排他制御の競合。"""
    store = InMemoryEventStore()
    await store.append("stream-1", [make_event()], expected_version=0)

    with pytest.raises(VersionConflictError) as exc_info:
        await store.append("stream-1", [make_event()], expected_version=0)
    assert exc_info.value.expected == 0
    assert exc_info.value.actual == 1


async def test_sequential_appends() -> None:
    """連続した追記。"""
    store = InMemoryEventStore()
    await store.append("stream-1", [make_event("E1")], expected_version=0)
    await store.append("stream-1", [make_event("E2")], expected_version=1)
    await store.append("stream-1", [make_event("E3")], expected_version=2)

    loaded = await store.load("stream-1")
    assert len(loaded) == 3


async def test_load_from_version() -> None:
    """指定バージョン以降の読み取り。"""
    store = InMemoryEventStore()
    await store.append("stream-1", [make_event("E1"), make_event("E2"), make_event("E3")], expected_version=0)

    loaded = await store.load_from("stream-1", from_version=1)
    assert len(loaded) == 2
    assert loaded[0].event_type == "E2"


async def test_exists_true() -> None:
    """ストリーム存在確認。"""
    store = InMemoryEventStore()
    await store.append("stream-1", [make_event()], expected_version=0)
    assert await store.exists("stream-1") is True


async def test_exists_false() -> None:
    """存在しないストリーム確認。"""
    store = InMemoryEventStore()
    assert await store.exists("nonexistent") is False


async def test_current_version() -> None:
    """現在のバージョン取得。"""
    store = InMemoryEventStore()
    assert await store.current_version("stream-1") == 0
    await store.append("stream-1", [make_event(), make_event()], expected_version=0)
    assert await store.current_version("stream-1") == 2


async def test_append_without_expected_version() -> None:
    """expected_version=None で楽観ロックなし追記。"""
    store = InMemoryEventStore()
    await store.append("stream-1", [make_event()])
    await store.append("stream-1", [make_event()])
    assert await store.current_version("stream-1") == 2


async def test_event_stream_id_assigned() -> None:
    """イベントに stream_id が自動付与されること。"""
    store = InMemoryEventStore()
    event = make_event()
    await store.append("stream-1", [event], expected_version=0)
    assert event.stream_id == "stream-1"


def test_version_conflict_error_message() -> None:
    """VersionConflictError のメッセージ。"""
    err = VersionConflictError(expected=0, actual=1)
    assert "expected=0" in str(err)
    assert "actual=1" in str(err)


def test_stream_not_found_error() -> None:
    """StreamNotFoundError のメッセージ。"""
    err = StreamNotFoundError("stream-x")
    assert err.stream_id == "stream-x"
    assert "stream-x" in str(err)
