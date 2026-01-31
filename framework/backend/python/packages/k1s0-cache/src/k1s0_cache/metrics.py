"""Cache metrics tracking."""

from __future__ import annotations


class CacheMetrics:
    """Tracks cache hit/miss statistics.

    Attributes:
        hit_count: Total number of cache hits.
        miss_count: Total number of cache misses.
        operation_count: Total number of get operations.
    """

    def __init__(self) -> None:
        self.hit_count: int = 0
        self.miss_count: int = 0
        self.operation_count: int = 0

    def record_hit(self) -> None:
        """Record a cache hit."""
        self.hit_count += 1
        self.operation_count += 1

    def record_miss(self) -> None:
        """Record a cache miss."""
        self.miss_count += 1
        self.operation_count += 1

    @property
    def hit_rate(self) -> float:
        """Return the cache hit rate as a float between 0.0 and 1.0.

        Returns:
            The hit rate, or 0.0 if no operations have been recorded.
        """
        if self.operation_count == 0:
            return 0.0
        return self.hit_count / self.operation_count
