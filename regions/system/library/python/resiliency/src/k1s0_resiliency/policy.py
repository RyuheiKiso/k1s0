from __future__ import annotations

from dataclasses import dataclass, field


@dataclass(frozen=True)
class RetryConfig:
    max_attempts: int = 3
    base_delay: float = 0.1
    max_delay: float = 5.0
    jitter: bool = True


@dataclass(frozen=True)
class CircuitBreakerConfig:
    failure_threshold: int = 5
    recovery_timeout: float = 30.0
    half_open_max_calls: int = 2


@dataclass(frozen=True)
class BulkheadConfig:
    max_concurrent_calls: int = 20
    max_wait_duration: float = 0.5


@dataclass(frozen=True)
class ResiliencyPolicy:
    retry: RetryConfig | None = None
    circuit_breaker: CircuitBreakerConfig | None = None
    bulkhead: BulkheadConfig | None = None
    timeout: float | None = None
