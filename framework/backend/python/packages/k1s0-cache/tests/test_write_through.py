"""Tests for the write-through pattern."""

from __future__ import annotations

from unittest.mock import AsyncMock, call

import pytest

from k1s0_cache.patterns.write_through import WriteThrough


@pytest.fixture
def mock_cache() -> AsyncMock:
    return AsyncMock()


class TestWriteThrough:
    @pytest.mark.asyncio
    async def test_writer_called_before_cache(self, mock_cache: AsyncMock) -> None:
        call_order: list[str] = []

        async def writer(k: str, v: str) -> None:
            call_order.append("writer")

        async def cache_set(k: str, v: str, ttl: int | None = None) -> None:
            call_order.append("cache")

        mock_cache.set = cache_set  # type: ignore[assignment]

        wt = WriteThrough(mock_cache, default_ttl=600)
        await wt.write("k", "v", writer)

        assert call_order == ["writer", "cache"]

    @pytest.mark.asyncio
    async def test_ttl_override(self, mock_cache: AsyncMock) -> None:
        writer = AsyncMock()
        mock_cache.set = AsyncMock()

        wt = WriteThrough(mock_cache, default_ttl=600)
        await wt.write("k", "v", writer, ttl=30)

        mock_cache.set.assert_awaited_once_with("k", "v", ttl=30)
