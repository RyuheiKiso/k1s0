"""InMemoryOutboxStore のユニットテスト"""

import pytest
from k1s0_outbox.in_memory_store import InMemoryOutboxStore
from k1s0_outbox.models import OutboxMessage, OutboxStatus


@pytest.mark.asyncio
async def test_save_and_fetch_pending() -> None:
    """保存したメッセージが fetch_pending で取得できること。"""
    store = InMemoryOutboxStore()
    msg = OutboxMessage(topic="events", payload=b"data")
    await store.save(msg)

    pending = await store.fetch_pending()
    assert len(pending) == 1
    assert pending[0].id == msg.id


@pytest.mark.asyncio
async def test_mark_published() -> None:
    """mark_published でステータスが PUBLISHED になること。"""
    store = InMemoryOutboxStore()
    msg = OutboxMessage(topic="events", payload=b"data")
    await store.save(msg)
    await store.mark_published(msg.id)

    pending = await store.fetch_pending()
    assert len(pending) == 0
    all_msgs = store.all_messages()
    assert all_msgs[0].status == OutboxStatus.PUBLISHED


@pytest.mark.asyncio
async def test_mark_failed() -> None:
    """mark_failed でステータスが FAILED になること。"""
    store = InMemoryOutboxStore()
    msg = OutboxMessage(topic="events", payload=b"data")
    await store.save(msg)
    await store.mark_failed(msg.id, "connection error")

    all_msgs = store.all_messages()
    assert all_msgs[0].status == OutboxStatus.FAILED
    assert all_msgs[0].error_message == "connection error"


@pytest.mark.asyncio
async def test_increment_retry() -> None:
    """increment_retry でリトライカウントが増加すること。"""
    store = InMemoryOutboxStore()
    msg = OutboxMessage(topic="events", payload=b"data")
    await store.save(msg)
    await store.increment_retry(msg.id)
    await store.increment_retry(msg.id)

    all_msgs = store.all_messages()
    assert all_msgs[0].retry_count == 2


@pytest.mark.asyncio
async def test_fetch_pending_limit() -> None:
    """fetch_pending の limit が適用されること。"""
    store = InMemoryOutboxStore()
    for i in range(5):
        await store.save(OutboxMessage(topic="events", payload=f"data{i}".encode()))

    pending = await store.fetch_pending(limit=3)
    assert len(pending) == 3
