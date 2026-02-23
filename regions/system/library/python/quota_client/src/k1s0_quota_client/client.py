"""Quota client implementations."""

from __future__ import annotations

import time
from abc import ABC, abstractmethod
from datetime import datetime, timedelta, timezone

from .config import QuotaClientConfig
from .exceptions import QuotaClientError, QuotaNotFoundError
from .model import QuotaPeriod, QuotaPolicy, QuotaStatus, QuotaUsage


class QuotaClient(ABC):
    """Abstract quota client."""

    @abstractmethod
    async def check(self, quota_id: str, amount: int) -> QuotaStatus: ...

    @abstractmethod
    async def increment(self, quota_id: str, amount: int) -> QuotaUsage: ...

    @abstractmethod
    async def get_usage(self, quota_id: str) -> QuotaUsage: ...

    @abstractmethod
    async def get_policy(self, quota_id: str) -> QuotaPolicy: ...


class HttpQuotaClient(QuotaClient):
    """HTTP-based quota client."""

    def __init__(self, config: QuotaClientConfig) -> None:
        self._config = config

    async def check(self, quota_id: str, amount: int) -> QuotaStatus:
        raise NotImplementedError("HTTP client requires a running server")

    async def increment(self, quota_id: str, amount: int) -> QuotaUsage:
        raise NotImplementedError("HTTP client requires a running server")

    async def get_usage(self, quota_id: str) -> QuotaUsage:
        raise NotImplementedError("HTTP client requires a running server")

    async def get_policy(self, quota_id: str) -> QuotaPolicy:
        raise NotImplementedError("HTTP client requires a running server")


class _UsageEntry:
    def __init__(
        self,
        quota_id: str,
        used: int,
        limit: int,
        period: QuotaPeriod,
        reset_at: datetime,
    ) -> None:
        self.quota_id = quota_id
        self.used = used
        self.limit = limit
        self.period = period
        self.reset_at = reset_at


class InMemoryQuotaClient(QuotaClient):
    """In-memory quota client for testing."""

    def __init__(self) -> None:
        self._usages: dict[str, _UsageEntry] = {}
        self._policies: dict[str, QuotaPolicy] = {}

    def set_policy(self, quota_id: str, policy: QuotaPolicy) -> None:
        """Register a policy for testing."""
        self._policies[quota_id] = policy

    def _get_or_create_usage(self, quota_id: str) -> _UsageEntry:
        if quota_id not in self._usages:
            policy = self._policies.get(quota_id)
            limit = policy.limit if policy else 1000
            period = policy.period if policy else QuotaPeriod.DAILY
            self._usages[quota_id] = _UsageEntry(
                quota_id=quota_id,
                used=0,
                limit=limit,
                period=period,
                reset_at=datetime.now(timezone.utc) + timedelta(days=1),
            )
        return self._usages[quota_id]

    async def check(self, quota_id: str, amount: int) -> QuotaStatus:
        usage = self._get_or_create_usage(quota_id)
        remaining = usage.limit - usage.used
        return QuotaStatus(
            allowed=amount <= remaining,
            remaining=remaining,
            limit=usage.limit,
            reset_at=usage.reset_at,
        )

    async def increment(self, quota_id: str, amount: int) -> QuotaUsage:
        usage = self._get_or_create_usage(quota_id)
        usage.used += amount
        return QuotaUsage(
            quota_id=usage.quota_id,
            used=usage.used,
            limit=usage.limit,
            period=usage.period,
            reset_at=usage.reset_at,
        )

    async def get_usage(self, quota_id: str) -> QuotaUsage:
        usage = self._get_or_create_usage(quota_id)
        return QuotaUsage(
            quota_id=usage.quota_id,
            used=usage.used,
            limit=usage.limit,
            period=usage.period,
            reset_at=usage.reset_at,
        )

    async def get_policy(self, quota_id: str) -> QuotaPolicy:
        if quota_id in self._policies:
            return self._policies[quota_id]
        return QuotaPolicy(
            quota_id=quota_id,
            limit=1000,
            period=QuotaPeriod.DAILY,
            reset_strategy="fixed",
        )


class _PolicyCacheEntry:
    def __init__(self, policy: QuotaPolicy, expires_at: float) -> None:
        self.policy = policy
        self.expires_at = expires_at


class CachedQuotaClient(QuotaClient):
    """Cached quota client with policy TTL."""

    def __init__(self, inner: QuotaClient, policy_ttl: timedelta) -> None:
        self._inner = inner
        self._policy_ttl_seconds = policy_ttl.total_seconds()
        self._cache: dict[str, _PolicyCacheEntry] = {}

    async def check(self, quota_id: str, amount: int) -> QuotaStatus:
        return await self._inner.check(quota_id, amount)

    async def increment(self, quota_id: str, amount: int) -> QuotaUsage:
        return await self._inner.increment(quota_id, amount)

    async def get_usage(self, quota_id: str) -> QuotaUsage:
        return await self._inner.get_usage(quota_id)

    async def get_policy(self, quota_id: str) -> QuotaPolicy:
        if quota_id in self._cache:
            entry = self._cache[quota_id]
            if time.monotonic() < entry.expires_at:
                return entry.policy
        policy = await self._inner.get_policy(quota_id)
        self._cache[quota_id] = _PolicyCacheEntry(
            policy=policy,
            expires_at=time.monotonic() + self._policy_ttl_seconds,
        )
        return policy
