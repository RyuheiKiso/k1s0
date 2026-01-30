"""Tests for the write-behind pattern."""

from __future__ import annotations

from unittest.mock import AsyncMock

import pytest

from k1s0_cache.patterns.write_behind import WriteBehind


@pytest.fixture
def mock_cache() -> AsyncMock:
    cache = AsyncMock()
    cache.set = AsyncMock()
    return cache


class TestWriteBehind:
    @pytest.mark.asyncio
    async def test_write_updates_cache_immediately(
        self, mock_cache: AsyncMock
    ) -> None:
        writer = AsyncMock()
        wb = WriteBehind(mock_cache, writer)

        await wb.write("k", "v")

        mock_cache.set.assert_awaited_once_with("k", "v")
        assert wb.stats.total_writes == 1

    @pytest.mark.asyncio
    async def test_buffer_accumulates(self, mock_cache: AsyncMock) -> None:
        writer = AsyncMock()
        wb = WriteBehind(mock_cache, writer, batch_size=10)

        await wb.write("k1", "v1")
        await wb.write("k2", "v2")

        assert len(wb._buffer) == 2
        writer.assert_not_awaited()

    @pytest.mark.asyncio
    async def test_flush_sends_batch(self, mock_cache: AsyncMock) -> None:
        writer = AsyncMock()
        wb = WriteBehind(mock_cache, writer, batch_size=10)

        await wb.write("k1", "v1")
        await wb.write("k2", "v2")
        await wb.flush()

        writer.assert_awaited_once_with([("k1", "v1"), ("k2", "v2")])
        assert wb.stats.total_flushes == 1
        assert len(wb._buffer) == 0

    @pytest.mark.asyncio
    async def test_flush_empty_buffer_is_noop(self, mock_cache: AsyncMock) -> None:
        writer = AsyncMock()
        wb = WriteBehind(mock_cache, writer)

        await wb.flush()

        writer.assert_not_awaited()
        assert wb.stats.total_flushes == 0

    @pytest.mark.asyncio
    async def test_flush_failure_increments_stats(
        self, mock_cache: AsyncMock
    ) -> None:
        writer = AsyncMock(side_effect=RuntimeError("db error"))
        wb = WriteBehind(mock_cache, writer, max_retries=1)

        await wb.write("k", "v")

        with pytest.raises(RuntimeError, match="db error"):
            await wb.flush()

        assert wb.stats.total_failures == 1
