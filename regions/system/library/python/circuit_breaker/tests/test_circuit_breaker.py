"""circuit_breaker library unit tests."""

import pytest
from k1s0_circuit_breaker import (
    CircuitBreaker,
    CircuitBreakerConfig,
    CircuitBreakerError,
    CircuitState,
)


async def test_starts_closed() -> None:
    cb = CircuitBreaker()
    assert cb.state == CircuitState.CLOSED


async def test_opens_after_threshold() -> None:
    cb = CircuitBreaker(CircuitBreakerConfig(failure_threshold=3))
    for _ in range(3):
        cb.record_failure()
    assert cb.state == CircuitState.OPEN


async def test_call_success() -> None:
    cb = CircuitBreaker()
    result = await cb.call(lambda: _async_return(42))
    assert result == 42
    assert cb.state == CircuitState.CLOSED


async def test_call_failure_propagates() -> None:
    cb = CircuitBreaker(CircuitBreakerConfig(failure_threshold=2))
    with pytest.raises(ValueError, match="boom"):
        await cb.call(lambda: _async_raise(ValueError("boom")))


async def test_open_circuit_rejects() -> None:
    cb = CircuitBreaker(CircuitBreakerConfig(failure_threshold=1))
    cb.record_failure()
    assert cb.state == CircuitState.OPEN
    with pytest.raises(CircuitBreakerError):
        await cb.call(lambda: _async_return(1))


async def test_half_open_after_timeout() -> None:
    cb = CircuitBreaker(CircuitBreakerConfig(failure_threshold=1, timeout=0.0))
    cb.record_failure()
    assert cb.state == CircuitState.HALF_OPEN


async def test_half_open_to_closed() -> None:
    cb = CircuitBreaker(
        CircuitBreakerConfig(failure_threshold=1, success_threshold=1, timeout=0.0)
    )
    cb.record_failure()
    assert cb.state == CircuitState.HALF_OPEN
    await cb.call(lambda: _async_return(1))
    assert cb.state == CircuitState.CLOSED


async def _async_return(value: object) -> object:
    return value


async def _async_raise(exc: Exception) -> None:
    raise exc
