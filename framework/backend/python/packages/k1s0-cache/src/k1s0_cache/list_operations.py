"""Abstract base class for list cache operations."""

from __future__ import annotations

from abc import ABC, abstractmethod


class ListOperations(ABC):
    """Port defining Redis list operations."""

    @abstractmethod
    async def lpush(self, key: str, value: str) -> int:
        """Push a value to the head of a list.

        Args:
            key: The list key.
            value: The value to push.

        Returns:
            The length of the list after the push.
        """

    @abstractmethod
    async def rpush(self, key: str, value: str) -> int:
        """Push a value to the tail of a list.

        Args:
            key: The list key.
            value: The value to push.

        Returns:
            The length of the list after the push.
        """

    @abstractmethod
    async def lpop(self, key: str) -> str | None:
        """Remove and return the first element of a list.

        Args:
            key: The list key.

        Returns:
            The removed element, or ``None`` if the list is empty.
        """

    @abstractmethod
    async def rpop(self, key: str) -> str | None:
        """Remove and return the last element of a list.

        Args:
            key: The list key.

        Returns:
            The removed element, or ``None`` if the list is empty.
        """

    @abstractmethod
    async def lrange(self, key: str, start: int, stop: int) -> list[str]:
        """Return a range of elements from a list.

        Args:
            key: The list key.
            start: Start index (inclusive).
            stop: Stop index (inclusive).

        Returns:
            A list of elements in the specified range.
        """
