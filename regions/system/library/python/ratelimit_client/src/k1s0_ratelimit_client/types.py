"""Rate limit types."""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime


@dataclass(frozen=True)
class RateLimitStatus:
    """Rate limit check result."""

    allowed: bool
    remaining: int
    reset_at: datetime
    retry_after_secs: int | None = None


@dataclass(frozen=True)
class RateLimitResult:
    """Rate limit consume result."""

    remaining: int
    reset_at: datetime


@dataclass(frozen=True)
class RateLimitPolicy:
    """Rate limit policy."""

    key: str
    limit: int
    window_secs: int
    algorithm: str
