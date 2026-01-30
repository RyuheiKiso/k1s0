"""Caching wrapper for policy repositories."""

from __future__ import annotations

import time

from k1s0_auth.policy.models import PolicyRule
from k1s0_auth.policy.repository import PolicyRepository


class CachedPolicyRepository(PolicyRepository):
    """Wraps a :class:`PolicyRepository` with an in-memory TTL cache.

    Args:
        inner: The underlying repository to delegate to.
        ttl: Cache time-to-live in seconds.
    """

    def __init__(self, inner: PolicyRepository, ttl: int = 300) -> None:
        self._inner = inner
        self._ttl = ttl
        self._cache: dict[str, tuple[float, list[PolicyRule]]] = {}

    async def get_rules(self, resource: str) -> list[PolicyRule]:
        """Return cached rules or fetch from the inner repository."""
        now = time.monotonic()
        entry = self._cache.get(resource)
        if entry is not None:
            cached_at, rules = entry
            if (now - cached_at) < self._ttl:
                return rules

        rules = await self._inner.get_rules(resource)
        self._cache[resource] = (now, rules)
        return rules
