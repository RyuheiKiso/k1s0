"""OIDC UserInfo endpoint client."""

from __future__ import annotations

import httpx
from pydantic import BaseModel, Field


class UserInfo(BaseModel):
    """Standard OIDC UserInfo claims.

    Attributes:
        sub: Subject identifier.
        name: Full name.
        email: Email address.
        email_verified: Whether the email has been verified.
        preferred_username: Preferred display name.
        picture: URL of the user's profile picture.
    """

    sub: str
    name: str | None = None
    email: str | None = None
    email_verified: bool = False
    preferred_username: str | None = None
    picture: str | None = None
    extra: dict[str, object] = Field(default_factory=dict)


class UserInfoClient:
    """Client for the OIDC UserInfo endpoint.

    Args:
        userinfo_endpoint: The UserInfo endpoint URL.
        http_client: Optional shared ``httpx.AsyncClient``.
    """

    def __init__(
        self,
        userinfo_endpoint: str,
        http_client: httpx.AsyncClient | None = None,
    ) -> None:
        self._endpoint = userinfo_endpoint
        self._http_client = http_client

    async def get_userinfo(self, access_token: str) -> UserInfo:
        """Fetch user information using an access token.

        Args:
            access_token: A valid OAuth2 access token.

        Returns:
            Parsed :class:`UserInfo`.

        Raises:
            httpx.HTTPStatusError: If the endpoint returns an error.
        """
        client = self._http_client or httpx.AsyncClient()
        owns_client = self._http_client is None
        try:
            response = await client.get(
                self._endpoint,
                headers={"Authorization": f"Bearer {access_token}"},
            )
            response.raise_for_status()
            data = response.json()
            known_keys = {"sub", "name", "email", "email_verified", "preferred_username", "picture"}
            extra = {k: v for k, v in data.items() if k not in known_keys}
            return UserInfo(**{k: v for k, v in data.items() if k in known_keys}, extra=extra)
        finally:
            if owns_client:
                await client.aclose()
