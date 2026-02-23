from __future__ import annotations

import asyncio

import pytest

from k1s0_resiliency import (
    BulkheadConfig,
    BulkheadFullError,
    CircuitBreakerConfig,
    CircuitBreakerOpenError,
    MaxRetriesExceededError,
    ResiliencyDecorator,
    ResiliencyPolicy,
    RetryConfig,
    with_resiliency,
)
from k1s0_resiliency import ResiliencyTimeoutError


async def test_execute_success() -> None:
    policy = ResiliencyPolicy()
    decorator = ResiliencyDecorator(policy)

    async def fn() -> int:
        return 42

    result = await decorator.execute(fn)
    assert result == 42


async def test_retry_success() -> None:
    policy = ResiliencyPolicy(
        retry=RetryConfig(max_attempts=3, base_delay=0.01, max_delay=0.1),
    )
    decorator = ResiliencyDecorator(policy)

    counter = 0

    async def fn() -> int:
        nonlocal counter
        counter += 1
        if counter < 3:
            raise RuntimeError("fail")
        return 99

    result = await decorator.execute(fn)
    assert result == 99
    assert counter == 3


async def test_max_retries_exceeded() -> None:
    policy = ResiliencyPolicy(
        retry=RetryConfig(max_attempts=2, base_delay=0.001, max_delay=0.01),
    )
    decorator = ResiliencyDecorator(policy)

    async def fn() -> int:
        raise RuntimeError("always fail")

    with pytest.raises(MaxRetriesExceededError) as exc_info:
        await decorator.execute(fn)
    assert exc_info.value.attempts == 2


async def test_timeout() -> None:
    policy = ResiliencyPolicy(timeout=0.05)
    decorator = ResiliencyDecorator(policy)

    async def fn() -> int:
        await asyncio.sleep(1)
        return 42

    with pytest.raises(ResiliencyTimeoutError):
        await decorator.execute(fn)


async def test_circuit_breaker_opens() -> None:
    policy = ResiliencyPolicy(
        circuit_breaker=CircuitBreakerConfig(
            failure_threshold=3,
            recovery_timeout=60.0,
            half_open_max_calls=1,
        ),
    )
    decorator = ResiliencyDecorator(policy)

    async def failing_fn() -> int:
        raise RuntimeError("fail")

    for _ in range(3):
        with pytest.raises(Exception):
            await decorator.execute(failing_fn)

    async def success_fn() -> int:
        return 42

    with pytest.raises(CircuitBreakerOpenError):
        await decorator.execute(success_fn)


async def test_bulkhead_full() -> None:
    policy = ResiliencyPolicy(
        bulkhead=BulkheadConfig(max_concurrent_calls=1, max_wait_duration=0.05),
    )
    decorator = ResiliencyDecorator(policy)

    started = asyncio.Event()
    done = asyncio.Event()

    async def long_fn() -> int:
        started.set()
        await done.wait()
        return 1

    task = asyncio.create_task(decorator.execute(long_fn))
    await started.wait()

    async def quick_fn() -> int:
        return 2

    with pytest.raises(BulkheadFullError):
        await decorator.execute(quick_fn)

    done.set()
    await task


async def test_with_resiliency_convenience() -> None:
    policy = ResiliencyPolicy()

    async def fn() -> int:
        return 42

    result = await with_resiliency(policy, fn)
    assert result == 42
