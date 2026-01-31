"""Tests for the cache-aside pattern."""

from __future__ import annotations

from unittest.mock import AsyncMock

import pytest

from k1s0_cache.patterns.cache_aside import CacheAside


@pytest.fixture
def mock_cache() -> AsyncMock:
    return AsyncMock()


class TestCacheAside:
    @pytest.mark.asyncio
    async def test_cache_hit(self, mock_cache: AsyncMock) -> None:
        mock_cache.get = AsyncMock(return_value="cached_value")
        loader = AsyncMock(return_value="loaded_value")

        aside = CacheAside(mock_cache)
        result = await aside.get_or_load("key", loader)

        assert result == "cached_value"
        loader.assert_not_awaited()

    @pytest.mark.asyncio
    async def test_cache_miss_calls_loader(self, mock_cache: AsyncMock) -> None:
        mock_cache.get = AsyncMock(return_value=None)
        mock_cache.set = AsyncMock()
        loader = AsyncMock(return_value="loaded_value")

        aside = CacheAside(mock_cache, default_ttl=300)
        result = await aside.get_or_load("key", loader)

        assert result == "loaded_value"
        loader.assert_awaited_once()
        mock_cache.set.assert_awaited_once_with("key", "loaded_value", ttl=300)

    @pytest.mark.asyncio
    async def test_ttl_override(self, mock_cache: AsyncMock) -> None:
        mock_cache.get = AsyncMock(return_value=None)
        mock_cache.set = AsyncMock()
        loader = AsyncMock(return_value="v")

        aside = CacheAside(mock_cache, default_ttl=300)
        await aside.get_or_load("key", loader, ttl=60)

        mock_cache.set.assert_awaited_once_with("key", "v", ttl=60)
