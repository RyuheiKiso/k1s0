"""OutboxProcessor のユニットテスト"""

import asyncio
from unittest.mock import AsyncMock, MagicMock

import pytest
from k1s0_outbox.in_memory_store import InMemoryOutboxStore
from k1s0_outbox.models import OutboxConfig, OutboxMessage, OutboxStatus, RetryConfig
from k1s0_outbox.processor import OutboxProcessor


@pytest.mark.asyncio
async def test_process_once_publishes_message() -> None:
    """process_once でメッセージが発行されること。"""
    store = InMemoryOutboxStore()
    msg = OutboxMessage(topic="events", payload=b"data")
    await store.save(msg)

    publisher = MagicMock()
    publisher.publish = MagicMock()

    processor = OutboxProcessor(store)
    count = await processor.process_once(publisher)

    assert count == 1
    publisher.publish.assert_called_once()
    all_msgs = store.all_messages()
    assert all_msgs[0].status == OutboxStatus.PUBLISHED


@pytest.mark.asyncio
async def test_process_once_with_async_publisher() -> None:
    """非同期 publisher でも動作すること。"""
    store = InMemoryOutboxStore()
    msg = OutboxMessage(topic="events", payload=b"data")
    await store.save(msg)

    publisher = MagicMock()
    publisher.publish = AsyncMock()

    processor = OutboxProcessor(store)
    await processor.process_once(publisher)

    publisher.publish.assert_awaited_once()


@pytest.mark.asyncio
async def test_process_once_increments_retry_on_failure() -> None:
    """発行失敗時にリトライカウントが増加すること。"""
    store = InMemoryOutboxStore()
    msg = OutboxMessage(topic="events", payload=b"data")
    await store.save(msg)

    publisher = MagicMock()
    publisher.publish = MagicMock(side_effect=Exception("publish failed"))

    config = OutboxConfig(retry=RetryConfig(max_retries=3))
    processor = OutboxProcessor(store, config)
    await processor.process_once(publisher)

    all_msgs = store.all_messages()
    assert all_msgs[0].retry_count == 1
    assert all_msgs[0].status == OutboxStatus.PENDING


@pytest.mark.asyncio
async def test_process_once_marks_failed_after_max_retries() -> None:
    """最大リトライ超過でメッセージが FAILED になること。"""
    store = InMemoryOutboxStore()
    msg = OutboxMessage(topic="events", payload=b"data", retry_count=3)
    await store.save(msg)

    publisher = MagicMock()
    publisher.publish = MagicMock(side_effect=Exception("publish failed"))

    config = OutboxConfig(retry=RetryConfig(max_retries=3))
    processor = OutboxProcessor(store, config)
    await processor.process_once(publisher)

    all_msgs = store.all_messages()
    assert all_msgs[0].status == OutboxStatus.FAILED


@pytest.mark.asyncio
async def test_process_once_empty_store() -> None:
    """空のストアで process_once を実行しても例外が発生しないこと。"""
    store = InMemoryOutboxStore()
    publisher = MagicMock()
    processor = OutboxProcessor(store)
    count = await processor.process_once(publisher)
    assert count == 0


@pytest.mark.asyncio
async def test_start_and_stop() -> None:
    """start でタスクが開始され stop で正常に停止すること。"""
    store = InMemoryOutboxStore()
    publisher = MagicMock()
    publisher.publish = MagicMock()

    config = OutboxConfig(polling_interval_seconds=0.05)
    processor = OutboxProcessor(store, config)

    await processor.start(publisher)
    assert processor._task is not None
    assert processor._running is True

    await asyncio.sleep(0.1)
    await processor.stop()

    assert processor._task is None
    assert processor._running is False


@pytest.mark.asyncio
async def test_stop_without_start() -> None:
    """start せずに stop を呼んでも例外が発生しないこと。"""
    store = InMemoryOutboxStore()
    processor = OutboxProcessor(store)
    await processor.stop()
    assert processor._task is None


@pytest.mark.asyncio
async def test_poll_loop_handles_exception_in_process_once() -> None:
    """_poll_loop が process_once の例外をログして継続すること。"""
    store = InMemoryOutboxStore()
    publisher = MagicMock()
    publisher.publish = MagicMock()

    config = OutboxConfig(polling_interval_seconds=0.05)
    processor = OutboxProcessor(store, config)

    call_count = 0
    original_process_once = processor.process_once

    async def flaky_process_once(pub):
        nonlocal call_count
        call_count += 1
        if call_count == 1:
            raise RuntimeError("transient error")
        return await original_process_once(pub)

    processor.process_once = flaky_process_once

    await processor.start(publisher)
    await asyncio.sleep(0.15)
    await processor.stop()

    assert call_count >= 2
