"""distributed_lock library unit tests."""

import pytest
from k1s0_distributed_lock import InMemoryDistributedLock, LockError


async def test_acquire_and_release() -> None:
    lock = InMemoryDistributedLock()
    guard = await lock.acquire("resource-1", ttl=10.0)
    assert guard.key == "resource-1"
    assert guard.token
    await lock.release(guard)


async def test_acquire_fails_when_locked() -> None:
    lock = InMemoryDistributedLock()
    await lock.acquire("resource-1", ttl=10.0)
    with pytest.raises(LockError, match="already locked"):
        await lock.acquire("resource-1", ttl=10.0)


async def test_is_locked() -> None:
    lock = InMemoryDistributedLock()
    assert await lock.is_locked("resource-1") is False
    await lock.acquire("resource-1", ttl=10.0)
    assert await lock.is_locked("resource-1") is True


async def test_release_then_reacquire() -> None:
    lock = InMemoryDistributedLock()
    guard = await lock.acquire("resource-1", ttl=10.0)
    await lock.release(guard)
    guard2 = await lock.acquire("resource-1", ttl=10.0)
    assert guard2.token != guard.token


async def test_release_wrong_token() -> None:
    lock = InMemoryDistributedLock()
    from k1s0_distributed_lock import LockGuard

    await lock.acquire("resource-1", ttl=10.0)
    fake_guard = LockGuard(key="resource-1", token="wrong-token")
    with pytest.raises(LockError, match="Token mismatch"):
        await lock.release(fake_guard)


async def test_release_nonexistent() -> None:
    lock = InMemoryDistributedLock()
    from k1s0_distributed_lock import LockGuard

    guard = LockGuard(key="no-such-key", token="x")
    with pytest.raises(LockError, match="not found"):
        await lock.release(guard)


async def test_expired_lock_can_be_reacquired() -> None:
    lock = InMemoryDistributedLock()
    await lock.acquire("resource-1", ttl=0.0)
    # TTL=0 means already expired
    guard2 = await lock.acquire("resource-1", ttl=10.0)
    assert guard2.token
