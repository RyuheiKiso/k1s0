"""Tests for the token bucket rate limiter."""

from __future__ import annotations

from unittest.mock import patch

import pytest

from k1s0_rate_limit.config import TokenBucketConfig
from k1s0_rate_limit.token_bucket import TokenBucket


class TestTokenBucket:
    """Tests for token bucket rate limiter behavior."""

    @pytest.mark.asyncio
    async def test_acquire_within_limit(self) -> None:
        bucket = TokenBucket(TokenBucketConfig(capacity=10, refill_rate=1.0))
        assert await bucket.try_acquire() is True
        assert bucket.stats().allowed == 1

    @pytest.mark.asyncio
    async def test_acquire_exhausts_tokens(self) -> None:
        bucket = TokenBucket(TokenBucketConfig(capacity=2, refill_rate=0.0))
        assert await bucket.try_acquire() is True
        assert await bucket.try_acquire() is True
        assert await bucket.try_acquire() is False
        stats = bucket.stats()
        assert stats.allowed == 2
        assert stats.rejected == 1
        assert stats.total == 3

    @pytest.mark.asyncio
    async def test_available_tokens_decrements(self) -> None:
        bucket = TokenBucket(TokenBucketConfig(capacity=5, refill_rate=0.0))
        assert bucket.available_tokens() == 5
        await bucket.try_acquire()
        assert bucket.available_tokens() == 4

    @pytest.mark.asyncio
    async def test_refill_restores_tokens(self) -> None:
        bucket = TokenBucket(TokenBucketConfig(capacity=5, refill_rate=100.0))
        # Exhaust all tokens
        for _ in range(5):
            await bucket.try_acquire()
        assert bucket.available_tokens() == 0

        # Simulate time passing for refill
        import time

        original_time = time.monotonic()
        with patch("k1s0_rate_limit.token_bucket.time") as mock_time:
            mock_time.monotonic.return_value = original_time + 1.0
            assert await bucket.try_acquire() is True

    @pytest.mark.asyncio
    async def test_refill_does_not_exceed_capacity(self) -> None:
        bucket = TokenBucket(TokenBucketConfig(capacity=5, refill_rate=1000.0))
        import time

        original_time = time.monotonic()
        with patch("k1s0_rate_limit.token_bucket.time") as mock_time:
            mock_time.monotonic.return_value = original_time + 10.0
            await bucket.try_acquire()
            # Even after lots of time, tokens should not exceed capacity
            assert bucket.available_tokens() <= 5

    @pytest.mark.asyncio
    async def test_time_until_available_when_tokens_exist(self) -> None:
        bucket = TokenBucket(TokenBucketConfig(capacity=5, refill_rate=1.0))
        assert bucket.time_until_available().total_seconds() == 0.0

    @pytest.mark.asyncio
    async def test_time_until_available_when_exhausted(self) -> None:
        bucket = TokenBucket(TokenBucketConfig(capacity=1, refill_rate=10.0))
        await bucket.try_acquire()
        wait = bucket.time_until_available()
        assert wait.total_seconds() > 0.0

    @pytest.mark.asyncio
    async def test_time_until_available_zero_refill_rate(self) -> None:
        bucket = TokenBucket(TokenBucketConfig(capacity=1, refill_rate=0.0))
        await bucket.try_acquire()
        wait = bucket.time_until_available()
        assert wait == wait.max

    @pytest.mark.asyncio
    async def test_default_config(self) -> None:
        bucket = TokenBucket()
        assert await bucket.try_acquire() is True
        assert bucket.available_tokens() >= 0

    def test_invalid_capacity(self) -> None:
        with pytest.raises(ValueError, match="capacity must be >= 1"):
            TokenBucketConfig(capacity=0)

    def test_invalid_refill_rate(self) -> None:
        with pytest.raises(ValueError, match="refill_rate must be >= 0.0"):
            TokenBucketConfig(refill_rate=-1.0)
