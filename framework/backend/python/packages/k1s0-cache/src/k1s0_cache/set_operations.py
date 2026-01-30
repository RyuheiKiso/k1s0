"""Abstract base class for set cache operations."""

from __future__ import annotations

from abc import ABC, abstractmethod


class SetOperations(ABC):
    """Port defining Redis set operations."""

    @abstractmethod
    async def sadd(self, key: str, *members: str) -> int:
        """Add one or more members to a set.

        Args:
            key: The set key.
            *members: Members to add.

        Returns:
            The number of members that were added (excluding already-present
            members).
        """

    @abstractmethod
    async def srem(self, key: str, *members: str) -> int:
        """Remove one or more members from a set.

        Args:
            key: The set key.
            *members: Members to remove.

        Returns:
            The number of members that were removed.
        """

    @abstractmethod
    async def smembers(self, key: str) -> set[str]:
        """Return all members of a set.

        Args:
            key: The set key.

        Returns:
            A set of all members.
        """

    @abstractmethod
    async def sismember(self, key: str, member: str) -> bool:
        """Check whether a member exists in a set.

        Args:
            key: The set key.
            member: The member to check.

        Returns:
            ``True`` if the member is in the set.
        """
