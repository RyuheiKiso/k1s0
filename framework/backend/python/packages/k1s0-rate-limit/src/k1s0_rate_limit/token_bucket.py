"""Token bucket rate limiter implementation."""

from __future__ import annotations

import asyncio
import time
from datetime import timedelta

from k1s0_rate_limit.config import TokenBucketConfig
from k1s0_rate_limit.rate_limiter import RateLimiter, RateLimitStats


class TokenBucket(RateLimiter):
    """Rate limiter using the token bucket algorithm.

    Tokens are added at a constant ``refill_rate`` up to ``capacity``.
    Each call to :meth:`try_acquire` consumes one token if available.

    Args:
        config: Token bucket configuration.

    Example::

        bucket = TokenBucket(TokenBucketConfig(capacity=10, refill_rate=1.0))
        if await bucket.try_acquire():
            process_request()
    """

    def __init__(self, config: TokenBucketConfig | None = None) -> None:
        cfg = config or TokenBucketConfig()
        self._capacity = cfg.capacity
        self._refill_rate = cfg.refill_rate
        self._tokens = float(cfg.capacity)
        self._last_refill = time.monotonic()
        self._lock = asyncio.Lock()
        self._allowed = 0
        self._rejected = 0

    def _refill(self) -> None:
        """Refill tokens based on elapsed time since last refill."""
        now = time.monotonic()
        elapsed = now - self._last_refill
        self._tokens = min(self._capacity, self._tokens + elapsed * self._refill_rate)
        self._last_refill = now

    async def try_acquire(self) -> bool:
        """Attempt to consume one token from the bucket.

        Returns:
            ``True`` if a token was available and consumed, ``False`` otherwise.
        """
        async with self._lock:
            self._refill()
            if self._tokens >= 1.0:
                self._tokens -= 1.0
                self._allowed += 1
                return True
            self._rejected += 1
            return False

    def time_until_available(self) -> timedelta:
        """Return estimated wait time until one token is available.

        Returns:
            A ``timedelta`` representing the wait duration.
        """
        if self._tokens >= 1.0:
            return timedelta()
        if self._refill_rate <= 0.0:
            return timedelta.max
        deficit = 1.0 - self._tokens
        return timedelta(seconds=deficit / self._refill_rate)

    def available_tokens(self) -> int:
        """Return the current integer number of available tokens.

        Returns:
            Non-negative integer count of whole tokens available.
        """
        return int(self._tokens)

    def stats(self) -> RateLimitStats:
        """Return a snapshot of current statistics.

        Returns:
            Rate limit statistics including allowed, rejected, and available.
        """
        return RateLimitStats(
            allowed=self._allowed,
            rejected=self._rejected,
            total=self._allowed + self._rejected,
            available=int(self._tokens),
        )
