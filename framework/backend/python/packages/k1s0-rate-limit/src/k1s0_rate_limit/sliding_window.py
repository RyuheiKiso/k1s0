"""Sliding window rate limiter implementation."""

from __future__ import annotations

import asyncio
import time
from collections import deque
from datetime import timedelta

from k1s0_rate_limit.config import SlidingWindowConfig
from k1s0_rate_limit.rate_limiter import RateLimiter, RateLimitStats


class SlidingWindow(RateLimiter):
    """Rate limiter using the sliding window algorithm.

    Tracks individual request timestamps within a rolling window.
    Requests are rejected once ``max_requests`` is reached within
    the ``window_size`` duration.

    Args:
        config: Sliding window configuration.

    Example::

        limiter = SlidingWindow(SlidingWindowConfig(
            window_size=timedelta(seconds=60),
            max_requests=100,
        ))
        if await limiter.try_acquire():
            process_request()
    """

    def __init__(self, config: SlidingWindowConfig | None = None) -> None:
        cfg = config or SlidingWindowConfig()
        self._window_size = cfg.window_size.total_seconds()
        self._max_requests = cfg.max_requests
        self._timestamps: deque[float] = deque()
        self._lock = asyncio.Lock()
        self._allowed = 0
        self._rejected = 0

    def _cleanup(self) -> None:
        """Remove timestamps outside the current window."""
        cutoff = time.monotonic() - self._window_size
        while self._timestamps and self._timestamps[0] < cutoff:
            self._timestamps.popleft()

    async def try_acquire(self) -> bool:
        """Attempt to record a request within the current window.

        Returns:
            ``True`` if the request is within the limit, ``False`` otherwise.
        """
        async with self._lock:
            self._cleanup()
            if len(self._timestamps) < self._max_requests:
                self._timestamps.append(time.monotonic())
                self._allowed += 1
                return True
            self._rejected += 1
            return False

    def time_until_available(self) -> timedelta:
        """Return estimated wait time until a slot opens in the window.

        Returns:
            A ``timedelta`` representing the wait duration.
        """
        self._cleanup()
        if len(self._timestamps) < self._max_requests:
            return timedelta()
        if not self._timestamps:
            return timedelta()
        # Oldest timestamp will expire first
        oldest = self._timestamps[0]
        wait = (oldest + self._window_size) - time.monotonic()
        return timedelta(seconds=max(0.0, wait))

    def available_tokens(self) -> int:
        """Return the number of requests still available in the current window.

        Returns:
            Non-negative count of remaining request slots.
        """
        self._cleanup()
        return max(0, self._max_requests - len(self._timestamps))

    def stats(self) -> RateLimitStats:
        """Return a snapshot of current statistics.

        Returns:
            Rate limit statistics including allowed, rejected, and available.
        """
        self._cleanup()
        return RateLimitStats(
            allowed=self._allowed,
            rejected=self._rejected,
            total=self._allowed + self._rejected,
            available=max(0, self._max_requests - len(self._timestamps)),
        )
