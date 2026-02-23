"""k1s0 distributed lock library."""

from .client import DistributedLock, LockError, LockGuard
from .memory import InMemoryDistributedLock

__all__ = [
    "DistributedLock",
    "InMemoryDistributedLock",
    "LockError",
    "LockGuard",
]
