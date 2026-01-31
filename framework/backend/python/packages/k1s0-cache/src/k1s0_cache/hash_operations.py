"""Abstract base class for hash cache operations."""

from __future__ import annotations

from abc import ABC, abstractmethod


class HashOperations(ABC):
    """Port defining Redis hash operations."""

    @abstractmethod
    async def hget(self, key: str, field: str) -> str | None:
        """Get a single field from a hash.

        Args:
            key: The hash key.
            field: The field name.

        Returns:
            The field value, or ``None`` if missing.
        """

    @abstractmethod
    async def hset(self, key: str, field: str, value: str) -> None:
        """Set a single field in a hash.

        Args:
            key: The hash key.
            field: The field name.
            value: The field value.
        """

    @abstractmethod
    async def hdel(self, key: str, field: str) -> bool:
        """Delete a field from a hash.

        Args:
            key: The hash key.
            field: The field name.

        Returns:
            ``True`` if the field was removed.
        """

    @abstractmethod
    async def hgetall(self, key: str) -> dict[str, str]:
        """Get all fields and values from a hash.

        Args:
            key: The hash key.

        Returns:
            A dictionary of field-value pairs.
        """
