"""Tests for CacheClient with mocked Redis."""

from __future__ import annotations

from unittest.mock import AsyncMock, MagicMock

import pytest

from k1s0_cache.client import CacheClient
from k1s0_cache.config import CacheConfig


@pytest.fixture
def config() -> CacheConfig:
    return CacheConfig(prefix="test")


@pytest.fixture
def mock_pool() -> MagicMock:
    pool = MagicMock()
    conn = AsyncMock()
    pool.get_connection = AsyncMock(return_value=conn)
    return pool


@pytest.fixture
def client(mock_pool: MagicMock, config: CacheConfig) -> CacheClient:
    return CacheClient(mock_pool, config)


def _conn(mock_pool: MagicMock) -> AsyncMock:
    """Helper to get the mock connection."""
    return mock_pool.get_connection.return_value


class TestPrefixedKey:
    def test_with_prefix(self, client: CacheClient) -> None:
        assert client._prefixed_key("foo") == "test:foo"

    def test_without_prefix(self, mock_pool: MagicMock) -> None:
        c = CacheClient(mock_pool, CacheConfig(prefix=""))
        assert c._prefixed_key("foo") == "foo"


class TestGetSetDelete:
    @pytest.mark.asyncio
    async def test_get(self, client: CacheClient, mock_pool: MagicMock) -> None:
        _conn(mock_pool).get = AsyncMock(return_value="bar")
        result = await client.get("foo")
        assert result == "bar"
        _conn(mock_pool).get.assert_awaited_once_with("test:foo")

    @pytest.mark.asyncio
    async def test_get_miss(self, client: CacheClient, mock_pool: MagicMock) -> None:
        _conn(mock_pool).get = AsyncMock(return_value=None)
        result = await client.get("missing")
        assert result is None

    @pytest.mark.asyncio
    async def test_set_default_ttl(
        self, client: CacheClient, mock_pool: MagicMock
    ) -> None:
        _conn(mock_pool).set = AsyncMock()
        await client.set("foo", "bar")
        _conn(mock_pool).set.assert_awaited_once_with("test:foo", "bar", ex=3600)

    @pytest.mark.asyncio
    async def test_set_custom_ttl(
        self, client: CacheClient, mock_pool: MagicMock
    ) -> None:
        _conn(mock_pool).set = AsyncMock()
        await client.set("foo", "bar", ttl=60)
        _conn(mock_pool).set.assert_awaited_once_with("test:foo", "bar", ex=60)

    @pytest.mark.asyncio
    async def test_delete_existing(
        self, client: CacheClient, mock_pool: MagicMock
    ) -> None:
        _conn(mock_pool).delete = AsyncMock(return_value=1)
        assert await client.delete("foo") is True

    @pytest.mark.asyncio
    async def test_delete_missing(
        self, client: CacheClient, mock_pool: MagicMock
    ) -> None:
        _conn(mock_pool).delete = AsyncMock(return_value=0)
        assert await client.delete("foo") is False

    @pytest.mark.asyncio
    async def test_exists(self, client: CacheClient, mock_pool: MagicMock) -> None:
        _conn(mock_pool).exists = AsyncMock(return_value=1)
        assert await client.exists("foo") is True

    @pytest.mark.asyncio
    async def test_incr(self, client: CacheClient, mock_pool: MagicMock) -> None:
        _conn(mock_pool).incrby = AsyncMock(return_value=5)
        assert await client.incr("counter", 2) == 5

    @pytest.mark.asyncio
    async def test_decr(self, client: CacheClient, mock_pool: MagicMock) -> None:
        _conn(mock_pool).decrby = AsyncMock(return_value=3)
        assert await client.decr("counter", 1) == 3
