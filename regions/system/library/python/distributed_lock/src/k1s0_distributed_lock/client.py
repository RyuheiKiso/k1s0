"""Distributed lock abstract client."""

from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass


@dataclass
class LockGuard:
    """Lock guard returned after acquiring a lock."""

    key: str
    token: str


class LockError(Exception):
    """Distributed lock error."""


class DistributedLock(ABC):
    """Abstract distributed lock client."""

    @abstractmethod
    async def acquire(self, key: str, ttl: float) -> LockGuard: ...

    @abstractmethod
    async def release(self, guard: LockGuard) -> None: ...

    @abstractmethod
    async def is_locked(self, key: str) -> bool: ...
