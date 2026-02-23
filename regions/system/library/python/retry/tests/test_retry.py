"""retry ライブラリのユニットテスト"""

import pytest
from k1s0_retry import (
    CircuitBreaker,
    CircuitBreakerState,
    RetryConfig,
    RetryError,
    with_retry,
)


def test_default_config() -> None:
    """デフォルト設定の確認。"""
    cfg = RetryConfig()
    assert cfg.max_attempts == 3
    assert cfg.initial_delay == 0.1
    assert cfg.max_delay == 30.0
    assert cfg.multiplier == 2.0
    assert cfg.jitter is True


def test_compute_delay_no_jitter() -> None:
    """ジッターなしの遅延計算。"""
    cfg = RetryConfig(initial_delay=0.1, multiplier=2.0, max_delay=30.0, jitter=False)
    assert cfg.compute_delay(0) == pytest.approx(0.1)
    assert cfg.compute_delay(1) == pytest.approx(0.2)
    assert cfg.compute_delay(2) == pytest.approx(0.4)


def test_compute_delay_capped() -> None:
    """遅延が max_delay で頭打ちになること。"""
    cfg = RetryConfig(initial_delay=10.0, multiplier=10.0, max_delay=30.0, jitter=False)
    assert cfg.compute_delay(5) == pytest.approx(30.0)


def test_compute_delay_with_jitter() -> None:
    """ジッター付きの遅延が許容範囲内であること。"""
    cfg = RetryConfig(initial_delay=1.0, multiplier=1.0, max_delay=30.0, jitter=True)
    for _ in range(50):
        delay = cfg.compute_delay(0)
        assert 0.9 <= delay <= 1.1


async def test_with_retry_success_first_try() -> None:
    """1回目で成功する場合。"""
    call_count = 0

    async def succeeds():
        nonlocal call_count
        call_count += 1
        return "ok"

    cfg = RetryConfig(max_attempts=3, initial_delay=0.0, jitter=False)
    result = await with_retry(cfg, succeeds)
    assert result == "ok"
    assert call_count == 1


async def test_with_retry_success_after_failures() -> None:
    """数回失敗後に成功する場合。"""
    call_count = 0

    async def fails_twice():
        nonlocal call_count
        call_count += 1
        if call_count < 3:
            raise ValueError("not yet")
        return "done"

    cfg = RetryConfig(max_attempts=5, initial_delay=0.0, jitter=False)
    result = await with_retry(cfg, fails_twice)
    assert result == "done"
    assert call_count == 3


async def test_with_retry_exhausted() -> None:
    """全リトライ失敗で RetryError。"""

    async def always_fails():
        raise RuntimeError("boom")

    cfg = RetryConfig(max_attempts=2, initial_delay=0.0, jitter=False)
    with pytest.raises(RetryError) as exc_info:
        await with_retry(cfg, always_fails)
    assert exc_info.value.attempts == 2
    assert isinstance(exc_info.value.last_error, RuntimeError)


def test_circuit_breaker_initial_state() -> None:
    """サーキットブレーカーの初期状態は CLOSED。"""
    cb = CircuitBreaker()
    assert cb.state == CircuitBreakerState.CLOSED
    assert cb.is_open() is False


def test_circuit_breaker_opens_on_failures() -> None:
    """failure_threshold 到達で OPEN になること。"""
    cb = CircuitBreaker(failure_threshold=3)
    for _ in range(3):
        cb.record_failure()
    assert cb.state == CircuitBreakerState.OPEN
    assert cb.is_open() is True


def test_circuit_breaker_half_open_after_timeout() -> None:
    """タイムアウト後に HALF_OPEN になること。"""
    cb = CircuitBreaker(failure_threshold=1, timeout=0.0)
    cb.record_failure()
    assert cb.is_open() is False
    assert cb.state == CircuitBreakerState.HALF_OPEN


def test_circuit_breaker_closes_on_successes() -> None:
    """HALF_OPEN 状態で success_threshold 到達で CLOSED に戻ること。"""
    cb = CircuitBreaker(failure_threshold=1, success_threshold=2, timeout=0.0)
    cb.record_failure()
    _ = cb.state  # trigger HALF_OPEN transition
    cb.record_success()
    assert cb.state == CircuitBreakerState.HALF_OPEN
    cb.record_success()
    assert cb.state == CircuitBreakerState.CLOSED


def test_circuit_breaker_success_resets_failure_count() -> None:
    """CLOSED 状態での成功は failure_count をリセットすること。"""
    cb = CircuitBreaker(failure_threshold=3)
    cb.record_failure()
    cb.record_failure()
    cb.record_success()
    cb.record_failure()
    assert cb.state == CircuitBreakerState.CLOSED
