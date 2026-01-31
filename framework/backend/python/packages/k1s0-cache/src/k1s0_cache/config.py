"""Cache configuration."""

from __future__ import annotations

from pydantic import BaseModel


class CacheConfig(BaseModel):
    """Configuration for the Redis cache connection.

    Attributes:
        host: Redis server hostname.
        port: Redis server port.
        db: Redis database number.
        prefix: Key prefix applied to all cache operations.
        pool_size: Maximum number of connections in the pool.
        default_ttl: Default time-to-live in seconds for cached entries.
        connect_timeout: Connection timeout in seconds.
    """

    host: str = "localhost"
    port: int = 6379
    db: int = 0
    prefix: str = ""
    pool_size: int = 10
    default_ttl: int = 3600
    connect_timeout: float = 5.0
