"""Token blacklist for revocation support."""

from __future__ import annotations

import time
from abc import ABC, abstractmethod


class TokenBlacklist(ABC):
    """Abstract interface for token blacklisting (revocation)."""

    @abstractmethod
    async def is_blacklisted(self, jti: str) -> bool:
        """Check whether a token ID has been blacklisted.

        Args:
            jti: The JWT ``jti`` (token ID) claim.

        Returns:
            ``True`` if the token is blacklisted.
        """

    @abstractmethod
    async def add(self, jti: str, expires_at: float) -> None:
        """Add a token ID to the blacklist.

        Args:
            jti: The JWT ``jti`` claim.
            expires_at: Unix timestamp when the token expires (for cleanup).
        """


class InMemoryBlacklist(TokenBlacklist):
    """In-memory token blacklist with automatic expiry cleanup."""

    def __init__(self) -> None:
        self._entries: dict[str, float] = {}

    async def is_blacklisted(self, jti: str) -> bool:
        """Check blacklist status, cleaning up expired entries first."""
        self._cleanup()
        return jti in self._entries

    async def add(self, jti: str, expires_at: float) -> None:
        """Add a token to the blacklist."""
        self._entries[jti] = expires_at

    def _cleanup(self) -> None:
        """Remove entries whose tokens have expired."""
        now = time.time()
        expired = [jti for jti, exp in self._entries.items() if exp <= now]
        for jti in expired:
            del self._entries[jti]
