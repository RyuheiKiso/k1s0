"""cache ライブラリのユニットテスト"""

import time

import pytest
from k1s0_cache import CacheError, CacheErrorCodes, InMemoryCacheClient


async def test_set_and_get() -> None:
    """値の保存と取得。"""
    client = InMemoryCacheClient()
    await client.set("key1", "value1")
    assert await client.get("key1") == "value1"


async def test_get_nonexistent_key() -> None:
    """存在しないキーは None。"""
    client = InMemoryCacheClient()
    assert await client.get("missing") is None


async def test_delete_existing_key() -> None:
    """既存キーの削除。"""
    client = InMemoryCacheClient()
    await client.set("key1", "value1")
    assert await client.delete("key1") is True
    assert await client.get("key1") is None


async def test_delete_nonexistent_key() -> None:
    """存在しないキーの削除は False。"""
    client = InMemoryCacheClient()
    assert await client.delete("missing") is False


async def test_exists_true() -> None:
    """存在するキーの確認。"""
    client = InMemoryCacheClient()
    await client.set("key1", "value1")
    assert await client.exists("key1") is True


async def test_exists_false() -> None:
    """存在しないキーの確認。"""
    client = InMemoryCacheClient()
    assert await client.exists("missing") is False


async def test_overwrite_value() -> None:
    """既存キーの上書き。"""
    client = InMemoryCacheClient()
    await client.set("key1", "v1")
    await client.set("key1", "v2")
    assert await client.get("key1") == "v2"


async def test_ttl_expiry() -> None:
    """TTL 期限切れで値が消えること。"""
    client = InMemoryCacheClient()
    await client.set("key1", "value1", ttl=0)
    time.sleep(0.01)
    assert await client.get("key1") is None


async def test_no_ttl_persists() -> None:
    """TTL なしの値は永続。"""
    client = InMemoryCacheClient()
    await client.set("key1", "value1")
    assert await client.get("key1") == "value1"


async def test_set_nx_succeeds() -> None:
    """set_nx はキーが存在しない場合に成功。"""
    client = InMemoryCacheClient()
    assert await client.set_nx("key1", "value1", ttl=60.0) is True
    assert await client.get("key1") == "value1"


async def test_set_nx_fails_on_existing() -> None:
    """set_nx はキーが存在する場合に失敗。"""
    client = InMemoryCacheClient()
    await client.set("key1", "value1")
    assert await client.set_nx("key1", "value2", ttl=60.0) is False
    assert await client.get("key1") == "value1"


def test_cache_error_format() -> None:
    """CacheError のフォーマット。"""
    err = CacheError(CacheErrorCodes.KEY_NOT_FOUND, "key missing")
    assert str(err) == "KEY_NOT_FOUND: key missing"
    assert err.code == CacheErrorCodes.KEY_NOT_FOUND
