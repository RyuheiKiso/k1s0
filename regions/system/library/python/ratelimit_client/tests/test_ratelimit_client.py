"""ratelimit_client library unit tests."""

import pytest

from k1s0_ratelimit_client import (
    InMemoryRateLimitClient,
    RateLimitError,
    RateLimitPolicy,
    RateLimitStatus,
)


async def test_check_allowed() -> None:
    client = InMemoryRateLimitClient()
    status = await client.check("test-key", 1)
    assert status.allowed is True
    assert status.remaining == 99
    assert status.retry_after_secs is None


async def test_check_denied() -> None:
    client = InMemoryRateLimitClient()
    client.set_policy(
        "limited",
        RateLimitPolicy(key="limited", limit=2, window_secs=60, algorithm="fixed_window"),
    )
    await client.consume("limited", 2)
    status = await client.check("limited", 1)
    assert status.allowed is False
    assert status.remaining == 0
    assert status.retry_after_secs == 60


async def test_consume_success() -> None:
    client = InMemoryRateLimitClient()
    result = await client.consume("test-key", 1)
    assert result.remaining == 99
    assert client.get_used_count("test-key") == 1


async def test_consume_exceeds_limit() -> None:
    client = InMemoryRateLimitClient()
    client.set_policy(
        "small",
        RateLimitPolicy(key="small", limit=1, window_secs=60, algorithm="token_bucket"),
    )
    await client.consume("small", 1)
    with pytest.raises(RateLimitError):
        await client.consume("small", 1)


async def test_get_limit_default() -> None:
    client = InMemoryRateLimitClient()
    policy = await client.get_limit("unknown")
    assert policy.limit == 100
    assert policy.window_secs == 3600
    assert policy.algorithm == "token_bucket"


async def test_get_limit_custom() -> None:
    client = InMemoryRateLimitClient()
    client.set_policy(
        "tenant:T1",
        RateLimitPolicy(
            key="tenant:T1", limit=50, window_secs=1800, algorithm="sliding_window"
        ),
    )
    policy = await client.get_limit("tenant:T1")
    assert policy.key == "tenant:T1"
    assert policy.limit == 50
    assert policy.algorithm == "sliding_window"


async def test_ratelimit_error_fields() -> None:
    err = RateLimitError("exceeded", code="LIMIT_EXCEEDED", retry_after_secs=30)
    assert err.code == "LIMIT_EXCEEDED"
    assert err.retry_after_secs == 30


async def test_status_fields() -> None:
    client = InMemoryRateLimitClient()
    status = await client.check("key", 5)
    assert isinstance(status, RateLimitStatus)
    assert status.allowed is True
    assert status.remaining == 95


async def test_multiple_consumes() -> None:
    client = InMemoryRateLimitClient()
    await client.consume("key", 10)
    await client.consume("key", 20)
    assert client.get_used_count("key") == 30
    status = await client.check("key", 1)
    assert status.remaining == 69
