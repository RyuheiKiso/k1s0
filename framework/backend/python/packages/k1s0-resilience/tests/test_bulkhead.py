"""Tests for the bulkhead module."""

from __future__ import annotations

import asyncio

import pytest

from k1s0_resilience.bulkhead import Bulkhead, BulkheadConfig
from k1s0_resilience.errors import ConcurrencyLimitError


@pytest.fixture(autouse=True)
def _reset_bulkheads() -> None:
    """Reset bulkhead instances between tests."""
    Bulkhead.reset_all()


class TestBulkhead:
    """Tests for named bulkhead isolation."""

    @pytest.mark.asyncio
    async def test_basic_execution(self) -> None:
        bh = Bulkhead(BulkheadConfig(name="test-svc", max_concurrent=2))

        async def op() -> str:
            return "ok"

        result = await bh.execute(op())
        assert result == "ok"

    @pytest.mark.asyncio
    async def test_isolation_between_bulkheads(self) -> None:
        """Two bulkheads with different names do not interfere."""
        bh_a = Bulkhead(BulkheadConfig(name="service-a", max_concurrent=1))
        bh_b = Bulkhead(BulkheadConfig(name="service-b", max_concurrent=1))

        barrier_a = asyncio.Event()

        async def blocking_a() -> str:
            await barrier_a.wait()
            return "a"

        async def fast_b() -> str:
            return "b"

        # Hold service-a's slot
        task_a = asyncio.create_task(bh_a.execute(blocking_a()))
        await asyncio.sleep(0.05)

        # service-b should still work
        result_b = await bh_b.execute(fast_b())
        assert result_b == "b"

        barrier_a.set()
        await task_a

    @pytest.mark.asyncio
    async def test_same_name_shares_semaphore(self) -> None:
        """Two Bulkhead instances with the same name share limits."""
        bh1 = Bulkhead(BulkheadConfig(name="shared", max_concurrent=1))
        bh2 = Bulkhead(BulkheadConfig(name="shared", max_concurrent=1))

        barrier = asyncio.Event()

        async def blocking() -> str:
            await barrier.wait()
            return "done"

        task = asyncio.create_task(bh1.execute(blocking()))
        await asyncio.sleep(0.05)

        with pytest.raises(ConcurrencyLimitError):
            async def noop() -> None:
                pass

            await bh2.execute(noop())

        barrier.set()
        await task

    def test_config_validation_empty_name(self) -> None:
        with pytest.raises(ValueError, match="must not be empty"):
            BulkheadConfig(name="", max_concurrent=5)

    def test_config_validation_max_concurrent(self) -> None:
        with pytest.raises(ValueError, match="must be >= 1"):
            BulkheadConfig(name="test", max_concurrent=0)
