"""Abstract base class for core cache operations."""

from __future__ import annotations

from abc import ABC, abstractmethod


class CacheOperations(ABC):
    """Port defining basic key-value cache operations."""

    @abstractmethod
    async def get(self, key: str) -> str | None:
        """Retrieve a value by key.

        Args:
            key: The cache key.

        Returns:
            The cached value, or ``None`` if the key does not exist.
        """

    @abstractmethod
    async def set(self, key: str, value: str, ttl: int | None = None) -> None:
        """Store a value under the given key.

        Args:
            key: The cache key.
            value: The value to store.
            ttl: Optional time-to-live in seconds. Uses the default TTL when
                ``None``.
        """

    @abstractmethod
    async def delete(self, key: str) -> bool:
        """Delete a key from the cache.

        Args:
            key: The cache key.

        Returns:
            ``True`` if the key was deleted, ``False`` if it did not exist.
        """

    @abstractmethod
    async def exists(self, key: str) -> bool:
        """Check whether a key exists.

        Args:
            key: The cache key.

        Returns:
            ``True`` if the key exists.
        """

    @abstractmethod
    async def incr(self, key: str, amount: int = 1) -> int:
        """Increment a numeric value.

        Args:
            key: The cache key.
            amount: Increment step.

        Returns:
            The value after incrementing.
        """

    @abstractmethod
    async def decr(self, key: str, amount: int = 1) -> int:
        """Decrement a numeric value.

        Args:
            key: The cache key.
            amount: Decrement step.

        Returns:
            The value after decrementing.
        """
