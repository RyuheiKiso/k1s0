"""Tests for the retry module."""

from __future__ import annotations

from unittest.mock import AsyncMock, patch

import pytest

from k1s0_resilience.retry import RetryConfig, RetryExecutor


class TestRetryConfig:
    """Tests for RetryConfig validation."""

    def test_default_config(self) -> None:
        config = RetryConfig()
        assert config.max_attempts == 3

    def test_invalid_max_attempts(self) -> None:
        with pytest.raises(ValueError, match="must be >= 1"):
            RetryConfig(max_attempts=0)


class TestRetryExecutor:
    """Tests for RetryExecutor."""

    @pytest.mark.asyncio
    async def test_succeeds_on_first_attempt(self) -> None:
        executor = RetryExecutor(RetryConfig(max_attempts=3))
        factory = AsyncMock(return_value="ok")

        result = await executor.execute(factory)
        assert result == "ok"
        assert factory.call_count == 1

    @pytest.mark.asyncio
    async def test_retries_then_succeeds(self) -> None:
        executor = RetryExecutor(
            RetryConfig(max_attempts=3, initial_interval=0.01, jitter_factor=0.0)
        )
        call_count = 0

        async def flaky() -> str:
            nonlocal call_count
            call_count += 1
            if call_count < 3:
                raise RuntimeError("fail")
            return "ok"

        result = await executor.execute(flaky)
        assert result == "ok"
        assert call_count == 3

    @pytest.mark.asyncio
    async def test_max_attempts_exceeded_raises_last(self) -> None:
        executor = RetryExecutor(
            RetryConfig(max_attempts=2, initial_interval=0.01, jitter_factor=0.0)
        )

        async def always_fail() -> str:
            raise RuntimeError("boom")

        with pytest.raises(RuntimeError, match="boom"):
            await executor.execute(always_fail)

    @pytest.mark.asyncio
    async def test_non_retryable_exception_not_retried(self) -> None:
        executor = RetryExecutor(
            RetryConfig(
                max_attempts=3,
                initial_interval=0.01,
                retryable_checker=lambda e: isinstance(e, RuntimeError),
            )
        )
        call_count = 0

        async def fail_value_error() -> str:
            nonlocal call_count
            call_count += 1
            raise ValueError("not retryable")

        with pytest.raises(ValueError):
            await executor.execute(fail_value_error)

        assert call_count == 1

    def test_exponential_backoff_calculation(self) -> None:
        executor = RetryExecutor(
            RetryConfig(
                initial_interval=1.0,
                multiplier=2.0,
                max_interval=60.0,
                jitter_factor=0.0,
            )
        )

        # With zero jitter, delays should be exact powers
        assert executor._calculate_delay(0) == 1.0  # 1 * 2^0
        assert executor._calculate_delay(1) == 2.0  # 1 * 2^1
        assert executor._calculate_delay(2) == 4.0  # 1 * 2^2
        assert executor._calculate_delay(3) == 8.0  # 1 * 2^3

    def test_backoff_respects_max_interval(self) -> None:
        executor = RetryExecutor(
            RetryConfig(
                initial_interval=1.0,
                multiplier=10.0,
                max_interval=5.0,
                jitter_factor=0.0,
            )
        )

        # 1 * 10^2 = 100, but capped at 5.0
        assert executor._calculate_delay(2) == 5.0

    def test_jitter_adds_randomness(self) -> None:
        executor = RetryExecutor(
            RetryConfig(
                initial_interval=1.0,
                multiplier=1.0,
                jitter_factor=0.5,
            )
        )

        delays = {executor._calculate_delay(0) for _ in range(20)}
        # With jitter, we should get varying delays
        assert len(delays) > 1
