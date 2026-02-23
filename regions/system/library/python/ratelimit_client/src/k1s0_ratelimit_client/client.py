"""Rate limit client."""

from __future__ import annotations

from abc import ABC, abstractmethod
from datetime import datetime, timedelta, timezone

from .exceptions import RateLimitError
from .types import RateLimitPolicy, RateLimitResult, RateLimitStatus

_DEFAULT_POLICY = RateLimitPolicy(
    key="default",
    limit=100,
    window_secs=3600,
    algorithm="token_bucket",
)


class RateLimitClient(ABC):
    """Abstract rate limit client."""

    @abstractmethod
    async def check(self, key: str, cost: int) -> RateLimitStatus: ...

    @abstractmethod
    async def consume(self, key: str, cost: int) -> RateLimitResult: ...

    @abstractmethod
    async def get_limit(self, key: str) -> RateLimitPolicy: ...


class InMemoryRateLimitClient(RateLimitClient):
    """In-memory rate limit client for testing."""

    def __init__(self) -> None:
        self._counters: dict[str, int] = {}
        self._policies: dict[str, RateLimitPolicy] = {}

    def set_policy(self, key: str, policy: RateLimitPolicy) -> None:
        """Set a rate limit policy for a key."""
        self._policies[key] = policy

    def _get_policy(self, key: str) -> RateLimitPolicy:
        return self._policies.get(key, _DEFAULT_POLICY)

    async def check(self, key: str, cost: int) -> RateLimitStatus:
        policy = self._get_policy(key)
        used = self._counters.get(key, 0)
        reset_at = datetime.now(timezone.utc) + timedelta(seconds=policy.window_secs)

        if used + cost > policy.limit:
            return RateLimitStatus(
                allowed=False,
                remaining=0,
                reset_at=reset_at,
                retry_after_secs=policy.window_secs,
            )

        return RateLimitStatus(
            allowed=True,
            remaining=policy.limit - used - cost,
            reset_at=reset_at,
        )

    async def consume(self, key: str, cost: int) -> RateLimitResult:
        policy = self._get_policy(key)
        used = self._counters.get(key, 0)

        if used + cost > policy.limit:
            raise RateLimitError(
                f"Rate limit exceeded for key: {key}",
                code="LIMIT_EXCEEDED",
                retry_after_secs=policy.window_secs,
            )

        self._counters[key] = used + cost
        remaining = policy.limit - (used + cost)
        reset_at = datetime.now(timezone.utc) + timedelta(seconds=policy.window_secs)

        return RateLimitResult(remaining=remaining, reset_at=reset_at)

    async def get_limit(self, key: str) -> RateLimitPolicy:
        return self._get_policy(key)

    def get_used_count(self, key: str) -> int:
        """Get the current used count for a key. For testing."""
        return self._counters.get(key, 0)
