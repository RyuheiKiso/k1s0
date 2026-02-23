"""Quota client tests."""

from datetime import timedelta

from k1s0_quota_client import (
    CachedQuotaClient,
    InMemoryQuotaClient,
    QuotaExceededError,
    QuotaPeriod,
    QuotaPolicy,
    QuotaStatus,
    QuotaUsage,
)


async def test_check_allowed() -> None:
    client = InMemoryQuotaClient()
    status = await client.check("q1", 100)
    assert status.allowed is True
    assert status.remaining == 1000
    assert status.limit == 1000


async def test_check_exceeded() -> None:
    client = InMemoryQuotaClient()
    await client.increment("q1", 900)
    status = await client.check("q1", 200)
    assert status.allowed is False
    assert status.remaining == 100


async def test_increment_accumulates() -> None:
    client = InMemoryQuotaClient()
    await client.increment("q1", 300)
    usage = await client.increment("q1", 200)
    assert usage.used == 500
    assert usage.limit == 1000


async def test_get_usage() -> None:
    client = InMemoryQuotaClient()
    await client.increment("q1", 100)
    usage = await client.get_usage("q1")
    assert usage.used == 100
    assert usage.quota_id == "q1"


async def test_get_policy_default() -> None:
    client = InMemoryQuotaClient()
    policy = await client.get_policy("q1")
    assert policy.quota_id == "q1"
    assert policy.limit == 1000
    assert policy.period == QuotaPeriod.DAILY
    assert policy.reset_strategy == "fixed"


async def test_get_policy_custom() -> None:
    client = InMemoryQuotaClient()
    client.set_policy(
        "q1",
        QuotaPolicy(
            quota_id="q1",
            limit=5000,
            period=QuotaPeriod.MONTHLY,
            reset_strategy="sliding",
        ),
    )
    policy = await client.get_policy("q1")
    assert policy.limit == 5000
    assert policy.period == QuotaPeriod.MONTHLY


async def test_cached_client_caches_policy() -> None:
    inner = InMemoryQuotaClient()
    cached = CachedQuotaClient(inner, policy_ttl=timedelta(minutes=1))
    p1 = await cached.get_policy("q1")
    inner.set_policy(
        "q1",
        QuotaPolicy(
            quota_id="q1",
            limit=9999,
            period=QuotaPeriod.HOURLY,
            reset_strategy="fixed",
        ),
    )
    p2 = await cached.get_policy("q1")
    assert p2.limit == p1.limit


async def test_cached_client_delegates_check() -> None:
    inner = InMemoryQuotaClient()
    cached = CachedQuotaClient(inner, policy_ttl=timedelta(minutes=1))
    status = await cached.check("q1", 100)
    assert status.allowed is True


async def test_cached_client_delegates_increment() -> None:
    inner = InMemoryQuotaClient()
    cached = CachedQuotaClient(inner, policy_ttl=timedelta(minutes=1))
    usage = await cached.increment("q1", 100)
    assert usage.used == 100


async def test_cached_client_delegates_get_usage() -> None:
    inner = InMemoryQuotaClient()
    cached = CachedQuotaClient(inner, policy_ttl=timedelta(minutes=1))
    await cached.increment("q1", 50)
    usage = await cached.get_usage("q1")
    assert usage.used == 50


async def test_quota_exceeded_error() -> None:
    err = QuotaExceededError("q1", 0)
    assert err.quota_id == "q1"
    assert err.remaining == 0
    assert "Quota exceeded" in str(err)


async def test_quota_status_fields() -> None:
    from datetime import datetime, timezone

    now = datetime.now(timezone.utc)
    status = QuotaStatus(allowed=True, remaining=500, limit=1000, reset_at=now)
    assert status.allowed is True
    assert status.remaining == 500
