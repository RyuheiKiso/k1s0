"""idempotency ライブラリのユニットテスト"""

from datetime import datetime, timedelta, timezone

import pytest
from k1s0_idempotency import (
    DuplicateKeyError,
    IdempotencyRecord,
    IdempotencyStatus,
    InMemoryIdempotencyStore,
)


def make_record(
    key: str,
    status: IdempotencyStatus = IdempotencyStatus.PENDING,
    expires_at: datetime | None = None,
) -> IdempotencyRecord:
    return IdempotencyRecord(key=key, status=status, expires_at=expires_at)


async def test_insert_and_get() -> None:
    """レコードの挿入と取得。"""
    store = InMemoryIdempotencyStore()
    record = make_record("req-1")
    await store.insert(record)
    result = await store.get("req-1")
    assert result is not None
    assert result.key == "req-1"
    assert result.status == IdempotencyStatus.PENDING


async def test_get_nonexistent_returns_none() -> None:
    """存在しないキーは None。"""
    store = InMemoryIdempotencyStore()
    assert await store.get("missing") is None


async def test_insert_duplicate_raises_error() -> None:
    """重複キーの挿入は DuplicateKeyError。"""
    store = InMemoryIdempotencyStore()
    await store.insert(make_record("req-1"))
    with pytest.raises(DuplicateKeyError) as exc_info:
        await store.insert(make_record("req-1"))
    assert exc_info.value.key == "req-1"


async def test_update_status() -> None:
    """ステータス更新。"""
    store = InMemoryIdempotencyStore()
    await store.insert(make_record("req-1"))
    await store.update("req-1", IdempotencyStatus.COMPLETED, body='{"ok":true}', code=200)
    result = await store.get("req-1")
    assert result is not None
    assert result.status == IdempotencyStatus.COMPLETED
    assert result.response_body == '{"ok":true}'
    assert result.status_code == 200
    assert result.completed_at is not None


async def test_delete_existing() -> None:
    """既存レコードの削除。"""
    store = InMemoryIdempotencyStore()
    await store.insert(make_record("req-1"))
    assert await store.delete("req-1") is True
    assert await store.get("req-1") is None


async def test_delete_nonexistent() -> None:
    """存在しないレコードの削除は False。"""
    store = InMemoryIdempotencyStore()
    assert await store.delete("missing") is False


async def test_expired_record_returns_none() -> None:
    """期限切れレコードは None。"""
    store = InMemoryIdempotencyStore()
    expired = make_record(
        "req-old",
        expires_at=datetime.now(timezone.utc) - timedelta(seconds=1),
    )
    store._records["req-old"] = expired  # bypass insert for expired setup
    assert await store.get("req-old") is None


def test_record_not_expired() -> None:
    """有効期限内のレコードは is_expired() == False。"""
    record = make_record(
        "req-1",
        expires_at=datetime.now(timezone.utc) + timedelta(hours=1),
    )
    assert record.is_expired() is False


def test_record_expired() -> None:
    """期限切れレコードは is_expired() == True。"""
    record = make_record(
        "req-old",
        expires_at=datetime.now(timezone.utc) - timedelta(seconds=1),
    )
    assert record.is_expired() is True


def test_record_no_expiry() -> None:
    """expires_at=None は期限なし。"""
    record = make_record("req-1")
    assert record.is_expired() is False


def test_status_values() -> None:
    """IdempotencyStatus の値確認。"""
    assert IdempotencyStatus.PENDING.value == "pending"
    assert IdempotencyStatus.COMPLETED.value == "completed"
    assert IdempotencyStatus.FAILED.value == "failed"


async def test_insert_after_expired_allows_reuse() -> None:
    """期限切れレコードの後に同一キーで insert 可能。"""
    store = InMemoryIdempotencyStore()
    expired = make_record(
        "req-reuse",
        expires_at=datetime.now(timezone.utc) - timedelta(seconds=1),
    )
    store._records["req-reuse"] = expired
    # get triggers cleanup
    assert await store.get("req-reuse") is None
    # Now insert should succeed
    await store.insert(make_record("req-reuse"))
    result = await store.get("req-reuse")
    assert result is not None
