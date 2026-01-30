"""k1s0-cache: Redis caching utilities for k1s0 Python services."""

from __future__ import annotations

from k1s0_cache.client import CacheClient
from k1s0_cache.config import CacheConfig
from k1s0_cache.errors import CacheError, ConnectionError, OperationError, SerializationError
from k1s0_cache.hash_operations import HashOperations
from k1s0_cache.health import CacheHealthChecker
from k1s0_cache.list_operations import ListOperations
from k1s0_cache.metrics import CacheMetrics
from k1s0_cache.operations import CacheOperations
from k1s0_cache.patterns.cache_aside import CacheAside
from k1s0_cache.patterns.ttl_refresh import TtlRefresh
from k1s0_cache.patterns.write_behind import WriteBehind
from k1s0_cache.patterns.write_through import WriteThrough
from k1s0_cache.pool import RedisPool
from k1s0_cache.set_operations import SetOperations

__all__ = [
    "CacheAside",
    "CacheClient",
    "CacheConfig",
    "CacheError",
    "CacheHealthChecker",
    "CacheMetrics",
    "CacheOperations",
    "ConnectionError",
    "HashOperations",
    "ListOperations",
    "OperationError",
    "RedisPool",
    "SerializationError",
    "SetOperations",
    "TtlRefresh",
    "WriteBehind",
    "WriteThrough",
]
