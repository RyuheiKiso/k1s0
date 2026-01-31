"""Tests for the sliding window rate limiter."""

from __future__ import annotations

from datetime import timedelta
from unittest.mock import patch

import pytest

from k1s0_rate_limit.config import SlidingWindowConfig
from k1s0_rate_limit.sliding_window import SlidingWindow


class TestSlidingWindow:
    """Tests for sliding window rate limiter behavior."""

    @pytest.mark.asyncio
    async def test_acquire_within_limit(self) -> None:
        limiter = SlidingWindow(SlidingWindowConfig(max_requests=5, window_size=timedelta(seconds=60)))
        assert await limiter.try_acquire() is True
        assert limiter.stats().allowed == 1

    @pytest.mark.asyncio
    async def test_acquire_exceeds_limit(self) -> None:
        limiter = SlidingWindow(SlidingWindowConfig(max_requests=2, window_size=timedelta(seconds=60)))
        assert await limiter.try_acquire() is True
        assert await limiter.try_acquire() is True
        assert await limiter.try_acquire() is False
        stats = limiter.stats()
        assert stats.allowed == 2
        assert stats.rejected == 1
        assert stats.total == 3

    @pytest.mark.asyncio
    async def test_available_tokens_decrements(self) -> None:
        limiter = SlidingWindow(SlidingWindowConfig(max_requests=5, window_size=timedelta(seconds=60)))
        assert limiter.available_tokens() == 5
        await limiter.try_acquire()
        assert limiter.available_tokens() == 4

    @pytest.mark.asyncio
    async def test_window_expiry_frees_slots(self) -> None:
        limiter = SlidingWindow(SlidingWindowConfig(max_requests=1, window_size=timedelta(seconds=1)))
        assert await limiter.try_acquire() is True
        assert await limiter.try_acquire() is False

        import time

        original_time = time.monotonic()
        with patch("k1s0_rate_limit.sliding_window.time") as mock_time:
            mock_time.monotonic.return_value = original_time + 2.0
            assert await limiter.try_acquire() is True

    @pytest.mark.asyncio
    async def test_time_until_available_when_slots_exist(self) -> None:
        limiter = SlidingWindow(SlidingWindowConfig(max_requests=5, window_size=timedelta(seconds=60)))
        assert limiter.time_until_available().total_seconds() == 0.0

    @pytest.mark.asyncio
    async def test_time_until_available_when_exhausted(self) -> None:
        limiter = SlidingWindow(SlidingWindowConfig(max_requests=1, window_size=timedelta(seconds=60)))
        await limiter.try_acquire()
        wait = limiter.time_until_available()
        assert wait.total_seconds() > 0.0

    @pytest.mark.asyncio
    async def test_default_config(self) -> None:
        limiter = SlidingWindow()
        assert await limiter.try_acquire() is True
        assert limiter.available_tokens() >= 0

    def test_invalid_max_requests(self) -> None:
        with pytest.raises(ValueError, match="max_requests must be >= 1"):
            SlidingWindowConfig(max_requests=0)

    def test_invalid_window_size(self) -> None:
        with pytest.raises(ValueError, match="window_size must be positive"):
            SlidingWindowConfig(window_size=timedelta(seconds=0))
