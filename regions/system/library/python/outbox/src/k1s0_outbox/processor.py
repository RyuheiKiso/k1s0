"""OutboxProcessor — asyncio Task ベースのポーリング処理"""

from __future__ import annotations

import asyncio
import contextlib
import logging
from typing import Protocol

from .models import OutboxConfig, OutboxMessage
from .store import OutboxStore


class _Publisher(Protocol):
    """Outbox メッセージを発行するプロトコル。"""

    def publish(self, message: OutboxMessage) -> None: ...


logger = logging.getLogger(__name__)


class OutboxProcessor:
    """Outbox メッセージを非同期でポーリングして発行するプロセッサー。"""

    def __init__(self, store: OutboxStore, config: OutboxConfig | None = None) -> None:
        self._store = store
        self._config = config or OutboxConfig()
        self._task: asyncio.Task[None] | None = None
        self._running = False

    async def start(self, publisher: _Publisher) -> None:
        """ポーリングタスクを開始する。

        Args:
            publisher: publish(message: OutboxMessage) -> None を持つオブジェクト
        """
        self._running = True
        self._task = asyncio.create_task(self._poll_loop(publisher))

    async def stop(self) -> None:
        """ポーリングタスクを停止する。"""
        self._running = False
        if self._task is not None:
            self._task.cancel()
            with contextlib.suppress(asyncio.CancelledError):
                await self._task
            self._task = None

    async def process_once(self, publisher: _Publisher) -> int:
        """1回のポーリングを実行して処理したメッセージ数を返す。

        Args:
            publisher: publish(message: OutboxMessage) -> None を持つオブジェクト

        Returns:
            処理したメッセージ数
        """
        messages = await self._store.fetch_pending(limit=self._config.batch_size)
        processed = 0
        for message in messages:
            await self._process_message(message, publisher)
            processed += 1
        return processed

    async def _process_message(self, message: OutboxMessage, publisher: _Publisher) -> None:
        """単一メッセージを処理する。"""
        try:
            if asyncio.iscoroutinefunction(publisher.publish):
                await publisher.publish(message)
            else:
                publisher.publish(message)
            await self._store.mark_published(message.id)
        except Exception as e:
            logger.warning(
                "Failed to publish outbox message",
                extra={"message_id": str(message.id), "error": str(e)},
            )
            if message.retry_count >= self._config.retry.max_retries:
                await self._store.mark_failed(message.id, str(e))
            else:
                await self._store.increment_retry(message.id)

    async def _poll_loop(self, publisher: _Publisher) -> None:
        """ポーリングループ。"""
        while self._running:
            try:
                await self.process_once(publisher)
            except Exception as e:
                logger.error("Outbox polling error", extra={"error": str(e)})
            await asyncio.sleep(self._config.polling_interval_seconds)
