"""Configuration dataclasses for rate limiters."""

from __future__ import annotations

from dataclasses import dataclass
from datetime import timedelta


@dataclass(frozen=True)
class TokenBucketConfig:
    """Configuration for the token bucket rate limiter.

    Attributes:
        capacity: Maximum number of tokens the bucket can hold.
        refill_rate: Number of tokens added per second.
    """

    capacity: int = 1000
    refill_rate: float = 100.0

    def __post_init__(self) -> None:
        if self.capacity < 1:
            msg = f"capacity must be >= 1, got {self.capacity}"
            raise ValueError(msg)
        if self.refill_rate < 0.0:
            msg = f"refill_rate must be >= 0.0, got {self.refill_rate}"
            raise ValueError(msg)


@dataclass(frozen=True)
class SlidingWindowConfig:
    """Configuration for the sliding window rate limiter.

    Attributes:
        window_size: Duration of the sliding window.
        max_requests: Maximum number of requests allowed within the window.
    """

    window_size: timedelta = timedelta(seconds=60)
    max_requests: int = 600

    def __post_init__(self) -> None:
        if self.window_size.total_seconds() <= 0:
            msg = f"window_size must be positive, got {self.window_size}"
            raise ValueError(msg)
        if self.max_requests < 1:
            msg = f"max_requests must be >= 1, got {self.max_requests}"
            raise ValueError(msg)
