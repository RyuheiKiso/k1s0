"""OIDC discovery client."""

from __future__ import annotations

from typing import Any

import httpx

from k1s0_auth.errors import DiscoveryError


class OidcDiscovery:
    """Fetches and caches the OpenID Connect discovery document.

    Args:
        issuer_url: The issuer base URL (e.g. ``https://auth.example.com``).
        http_client: Optional shared ``httpx.AsyncClient``.
    """

    def __init__(
        self,
        issuer_url: str,
        http_client: httpx.AsyncClient | None = None,
    ) -> None:
        self._issuer_url = issuer_url.rstrip("/")
        self._http_client = http_client
        self._cache: dict[str, Any] | None = None

    async def discover(self) -> dict[str, Any]:
        """Fetch the OIDC discovery document.

        Returns:
            The parsed JSON discovery document.

        Raises:
            DiscoveryError: If the fetch fails.
        """
        if self._cache is not None:
            return self._cache

        url = f"{self._issuer_url}/.well-known/openid-configuration"
        client = self._http_client or httpx.AsyncClient()
        owns_client = self._http_client is None
        try:
            response = await client.get(url)
            response.raise_for_status()
            self._cache = response.json()
            return self._cache  # type: ignore[return-value]
        except httpx.HTTPError as exc:
            raise DiscoveryError(f"Failed to fetch discovery document: {exc}") from exc
        finally:
            if owns_client:
                await client.aclose()
