"""k1s0-resilience: Resilience patterns for k1s0 Python services."""

from __future__ import annotations

from k1s0_resilience.bulkhead import Bulkhead, BulkheadConfig
from k1s0_resilience.circuit_breaker import CircuitBreaker, CircuitBreakerConfig, CircuitState
from k1s0_resilience.concurrency import ConcurrencyConfig, ConcurrencyLimiter, ConcurrencyMetrics
from k1s0_resilience.errors import (
    CircuitOpenError,
    ConcurrencyLimitError,
    ResilienceError,
    TimeoutError,
)
from k1s0_resilience.retry import RetryConfig, RetryExecutor
from k1s0_resilience.timeout import TimeoutConfig, TimeoutGuard

__all__ = [
    "Bulkhead",
    "BulkheadConfig",
    "CircuitBreaker",
    "CircuitBreakerConfig",
    "CircuitOpenError",
    "CircuitState",
    "ConcurrencyConfig",
    "ConcurrencyLimitError",
    "ConcurrencyLimiter",
    "ConcurrencyMetrics",
    "ResilienceError",
    "RetryConfig",
    "RetryExecutor",
    "TimeoutConfig",
    "TimeoutError",
    "TimeoutGuard",
]
