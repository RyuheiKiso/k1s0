"""Refresh token management."""

from __future__ import annotations

import time
import uuid
from abc import ABC, abstractmethod

import jwt


class RefreshTokenStore(ABC):
    """Abstract store for refresh token state."""

    @abstractmethod
    async def store(self, token_id: str, sub: str, expires_at: float) -> None:
        """Persist a refresh token.

        Args:
            token_id: Unique token identifier.
            sub: Subject the token belongs to.
            expires_at: Unix timestamp of expiry.
        """

    @abstractmethod
    async def get_subject(self, token_id: str) -> str | None:
        """Retrieve the subject for a refresh token.

        Args:
            token_id: Unique token identifier.

        Returns:
            The subject string, or ``None`` if not found or expired.
        """

    @abstractmethod
    async def revoke(self, token_id: str) -> None:
        """Revoke a refresh token.

        Args:
            token_id: Unique token identifier.
        """


class InMemoryRefreshTokenStore(RefreshTokenStore):
    """In-memory refresh token store for testing."""

    def __init__(self) -> None:
        self._tokens: dict[str, tuple[str, float]] = {}

    async def store(self, token_id: str, sub: str, expires_at: float) -> None:
        """Store the token."""
        self._tokens[token_id] = (sub, expires_at)

    async def get_subject(self, token_id: str) -> str | None:
        """Get the subject if the token exists and is not expired."""
        entry = self._tokens.get(token_id)
        if entry is None:
            return None
        sub, expires_at = entry
        if time.time() > expires_at:
            del self._tokens[token_id]
            return None
        return sub

    async def revoke(self, token_id: str) -> None:
        """Remove the token."""
        self._tokens.pop(token_id, None)


class RefreshTokenManager:
    """Issues, verifies, and revokes refresh tokens.

    Args:
        store: The underlying refresh token store.
        secret_key: HMAC secret for signing refresh tokens.
        token_ttl: Token time-to-live in seconds.
    """

    def __init__(
        self,
        store: RefreshTokenStore,
        secret_key: str,
        token_ttl: int = 86400,
    ) -> None:
        self._store = store
        self._secret_key = secret_key
        self._token_ttl = token_ttl

    async def issue(self, sub: str) -> str:
        """Issue a new refresh token for the given subject.

        Args:
            sub: Subject identifier.

        Returns:
            The signed refresh token string.
        """
        token_id = str(uuid.uuid4())
        now = time.time()
        expires_at = now + self._token_ttl

        await self._store.store(token_id, sub, expires_at)

        payload = {
            "jti": token_id,
            "sub": sub,
            "iat": now,
            "exp": expires_at,
        }
        return jwt.encode(payload, self._secret_key, algorithm="HS256")

    async def verify(self, token: str) -> str:
        """Verify a refresh token and return the subject.

        Args:
            token: The signed refresh token string.

        Returns:
            The subject identifier.

        Raises:
            jwt.InvalidTokenError: If the token is invalid or expired.
            ValueError: If the token has been revoked.
        """
        payload = jwt.decode(token, self._secret_key, algorithms=["HS256"])
        token_id: str = payload["jti"]

        sub = await self._store.get_subject(token_id)
        if sub is None:
            raise ValueError("Refresh token has been revoked or expired")

        return sub

    async def revoke(self, token: str) -> None:
        """Revoke a refresh token.

        Args:
            token: The signed refresh token string.
        """
        payload = jwt.decode(
            token, self._secret_key, algorithms=["HS256"], options={"verify_exp": False}
        )
        await self._store.revoke(payload["jti"])
