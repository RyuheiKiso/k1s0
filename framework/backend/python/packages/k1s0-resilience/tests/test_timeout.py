"""Tests for the timeout module."""

from __future__ import annotations

import asyncio

import pytest

from k1s0_resilience.errors import TimeoutError
from k1s0_resilience.timeout import TimeoutConfig, TimeoutGuard


class TestTimeoutConfig:
    """Tests for TimeoutConfig validation."""

    def test_default_config(self) -> None:
        config = TimeoutConfig()
        assert config.duration_seconds == 30.0

    def test_config_below_minimum_raises(self) -> None:
        with pytest.raises(ValueError, match="must be >= 0.1"):
            TimeoutConfig(duration_seconds=0.01)

    def test_config_above_maximum_raises(self) -> None:
        with pytest.raises(ValueError, match="must be <= 300"):
            TimeoutConfig(duration_seconds=500.0)

    def test_config_at_boundaries(self) -> None:
        TimeoutConfig(duration_seconds=0.1)
        TimeoutConfig(duration_seconds=300.0)


class TestTimeoutGuard:
    """Tests for TimeoutGuard execution."""

    @pytest.mark.asyncio
    async def test_completes_within_timeout(self) -> None:
        guard = TimeoutGuard(TimeoutConfig(duration_seconds=1.0))

        async def fast_op() -> str:
            return "done"

        result = await guard.execute(fast_op())
        assert result == "done"

    @pytest.mark.asyncio
    async def test_raises_timeout_error(self) -> None:
        guard = TimeoutGuard(TimeoutConfig(duration_seconds=0.1))

        async def slow_op() -> str:
            await asyncio.sleep(10)
            return "never"

        with pytest.raises(TimeoutError, match="timed out"):
            await guard.execute(slow_op())

    @pytest.mark.asyncio
    async def test_timeout_error_is_retryable(self) -> None:
        guard = TimeoutGuard(TimeoutConfig(duration_seconds=0.1))

        async def slow_op() -> None:
            await asyncio.sleep(10)

        with pytest.raises(TimeoutError) as exc_info:
            await guard.execute(slow_op())

        assert exc_info.value.is_retryable is True
