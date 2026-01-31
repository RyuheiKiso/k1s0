"""Tests for the rate limit middleware."""

from __future__ import annotations

import pytest

from k1s0_rate_limit.config import TokenBucketConfig
from k1s0_rate_limit.errors import RateLimitExceededError
from k1s0_rate_limit.middleware import RateLimitMiddleware
from k1s0_rate_limit.token_bucket import TokenBucket


class TestRateLimitMiddleware:
    """Tests for middleware integration."""

    @pytest.mark.asyncio
    async def test_check_passes_when_within_limit(self) -> None:
        limiter = TokenBucket(TokenBucketConfig(capacity=10, refill_rate=0.0))
        middleware = RateLimitMiddleware(limiter)
        await middleware.check()  # should not raise

    @pytest.mark.asyncio
    async def test_check_raises_when_exceeded(self) -> None:
        limiter = TokenBucket(TokenBucketConfig(capacity=1, refill_rate=0.0))
        middleware = RateLimitMiddleware(limiter)
        await middleware.check()  # consumes the single token

        with pytest.raises(RateLimitExceededError) as exc_info:
            await middleware.check()

        assert exc_info.value.retry_after is not None
        assert exc_info.value.error_code == "rate_limit.exceeded"
        assert exc_info.value.is_retryable is True
