"""Tests for the concurrency module."""

from __future__ import annotations

import asyncio

import pytest

from k1s0_resilience.concurrency import ConcurrencyConfig, ConcurrencyLimiter
from k1s0_resilience.errors import ConcurrencyLimitError


class TestConcurrencyLimiter:
    """Tests for ConcurrencyLimiter."""

    @pytest.mark.asyncio
    async def test_allows_within_limit(self) -> None:
        limiter = ConcurrencyLimiter(ConcurrencyConfig(max_concurrent=2))

        async def op() -> str:
            return "ok"

        result = await limiter.execute(op())
        assert result == "ok"

    @pytest.mark.asyncio
    async def test_rejects_when_limit_exceeded(self) -> None:
        limiter = ConcurrencyLimiter(ConcurrencyConfig(max_concurrent=1))
        barrier = asyncio.Event()

        async def blocking_op() -> str:
            await barrier.wait()
            return "done"

        # Start one task that holds the semaphore
        task = asyncio.create_task(limiter.execute(blocking_op()))
        await asyncio.sleep(0.05)  # let it acquire

        # Second call should be rejected
        with pytest.raises(ConcurrencyLimitError):
            async def another_op() -> str:
                return "never"

            await limiter.execute(another_op())

        # Cleanup
        barrier.set()
        await task

    @pytest.mark.asyncio
    async def test_metrics_tracking(self) -> None:
        limiter = ConcurrencyLimiter(ConcurrencyConfig(max_concurrent=1))
        barrier = asyncio.Event()

        async def blocking_op() -> str:
            await barrier.wait()
            return "done"

        task = asyncio.create_task(limiter.execute(blocking_op()))
        await asyncio.sleep(0.05)

        assert limiter.metrics.active_count == 1

        # Trigger a rejection
        with pytest.raises(ConcurrencyLimitError):
            async def noop() -> None:
                pass

            await limiter.execute(noop())

        assert limiter.metrics.rejected_count == 1

        barrier.set()
        await task
        assert limiter.metrics.active_count == 0

    def test_config_validation(self) -> None:
        with pytest.raises(ValueError, match="must be >= 1"):
            ConcurrencyConfig(max_concurrent=0)
