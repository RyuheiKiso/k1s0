"""Tests for the circuit breaker module."""

from __future__ import annotations

import time
from unittest.mock import patch

import pytest

from k1s0_resilience.circuit_breaker import CircuitBreaker, CircuitBreakerConfig, CircuitState
from k1s0_resilience.errors import CircuitOpenError


class TestCircuitBreaker:
    """Tests for circuit breaker state transitions."""

    @pytest.mark.asyncio
    async def test_starts_closed(self) -> None:
        cb = CircuitBreaker(CircuitBreakerConfig())
        assert cb.state == CircuitState.CLOSED

    @pytest.mark.asyncio
    async def test_successful_call(self) -> None:
        cb = CircuitBreaker(CircuitBreakerConfig())

        async def ok() -> str:
            return "ok"

        result = await cb.execute(ok())
        assert result == "ok"
        assert cb.state == CircuitState.CLOSED

    @pytest.mark.asyncio
    async def test_closed_to_open_on_failures(self) -> None:
        cb = CircuitBreaker(CircuitBreakerConfig(failure_threshold=3))

        async def fail() -> None:
            raise RuntimeError("boom")

        for _ in range(3):
            with pytest.raises(RuntimeError):
                await cb.execute(fail())

        assert cb.state == CircuitState.OPEN

    @pytest.mark.asyncio
    async def test_open_rejects_calls(self) -> None:
        cb = CircuitBreaker(CircuitBreakerConfig(failure_threshold=1))

        async def fail() -> None:
            raise RuntimeError("boom")

        with pytest.raises(RuntimeError):
            await cb.execute(fail())

        assert cb.state == CircuitState.OPEN

        with pytest.raises(CircuitOpenError):
            async def ok() -> str:
                return "ok"

            await cb.execute(ok())

        assert cb.rejected_count == 1

    @pytest.mark.asyncio
    async def test_open_to_half_open_after_timeout(self) -> None:
        cb = CircuitBreaker(CircuitBreakerConfig(failure_threshold=1, reset_timeout=0.1))

        async def fail() -> None:
            raise RuntimeError("boom")

        with pytest.raises(RuntimeError):
            await cb.execute(fail())

        assert cb.state == CircuitState.OPEN

        # Simulate time passing
        with patch("k1s0_resilience.circuit_breaker.time") as mock_time:
            mock_time.monotonic.return_value = time.monotonic() + 1.0
            assert cb.state == CircuitState.HALF_OPEN

    @pytest.mark.asyncio
    async def test_half_open_to_closed_on_successes(self) -> None:
        cb = CircuitBreaker(
            CircuitBreakerConfig(failure_threshold=1, success_threshold=2, reset_timeout=0.1)
        )

        async def fail() -> None:
            raise RuntimeError("boom")

        async def ok() -> str:
            return "ok"

        # Trip to OPEN
        with pytest.raises(RuntimeError):
            await cb.execute(fail())

        # Force to HALF_OPEN via time mock
        with patch("k1s0_resilience.circuit_breaker.time") as mock_time:
            mock_time.monotonic.return_value = time.monotonic() + 1.0
            assert cb.state == CircuitState.HALF_OPEN

            # Two successes should close it
            await cb.execute(ok())
            await cb.execute(ok())
            assert cb.state == CircuitState.CLOSED

    @pytest.mark.asyncio
    async def test_half_open_to_open_on_failure(self) -> None:
        cb = CircuitBreaker(CircuitBreakerConfig(failure_threshold=1, reset_timeout=0.1))

        async def fail() -> None:
            raise RuntimeError("boom")

        # Trip to OPEN
        with pytest.raises(RuntimeError):
            await cb.execute(fail())

        # Force to HALF_OPEN
        with patch("k1s0_resilience.circuit_breaker.time") as mock_time:
            mock_time.monotonic.return_value = time.monotonic() + 1.0
            assert cb.state == CircuitState.HALF_OPEN

            # Failure sends back to OPEN
            with pytest.raises(RuntimeError):
                await cb.execute(fail())

            # Need to also mock for the state check
            assert cb._state == CircuitState.OPEN

    @pytest.mark.asyncio
    async def test_custom_failure_predicate(self) -> None:
        """Only ValueError counts as a failure."""
        cb = CircuitBreaker(
            CircuitBreakerConfig(
                failure_threshold=1,
                failure_predicate=lambda e: isinstance(e, ValueError),
            )
        )

        async def type_error() -> None:
            raise TypeError("not a failure")

        async def value_error() -> None:
            raise ValueError("a failure")

        # TypeError should not trip the breaker
        with pytest.raises(TypeError):
            await cb.execute(type_error())
        assert cb.state == CircuitState.CLOSED

        # ValueError should trip it
        with pytest.raises(ValueError):
            await cb.execute(value_error())
        assert cb.state == CircuitState.OPEN

    @pytest.mark.asyncio
    async def test_state_transition_count(self) -> None:
        cb = CircuitBreaker(CircuitBreakerConfig(failure_threshold=1, reset_timeout=0.1))

        async def fail() -> None:
            raise RuntimeError("boom")

        with pytest.raises(RuntimeError):
            await cb.execute(fail())

        # CLOSED -> OPEN = 1 transition
        assert cb.state_transition_count == 1
