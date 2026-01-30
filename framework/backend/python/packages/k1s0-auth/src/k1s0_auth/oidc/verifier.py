"""OIDC-aware JWT verifier that auto-discovers JWKS URI."""

from __future__ import annotations

import httpx

from k1s0_auth.jwt.claims import Claims
from k1s0_auth.jwt.config import JwtVerifierConfig
from k1s0_auth.jwt.verifier import JwtVerifier
from k1s0_auth.oidc.discovery import OidcDiscovery


class OidcJwtVerifier:
    """JWT verifier that uses OIDC discovery to resolve the JWKS URI.

    Args:
        issuer_url: The OIDC issuer base URL.
        audience: Expected ``aud`` claim value.
        http_client: Optional shared ``httpx.AsyncClient``.
    """

    def __init__(
        self,
        issuer_url: str,
        audience: str,
        http_client: httpx.AsyncClient | None = None,
    ) -> None:
        self._issuer_url = issuer_url
        self._audience = audience
        self._http_client = http_client
        self._discovery = OidcDiscovery(issuer_url, http_client)
        self._verifier: JwtVerifier | None = None

    async def verify(self, token: str) -> Claims:
        """Verify a JWT token using OIDC-discovered JWKS.

        Args:
            token: The raw JWT string.

        Returns:
            Parsed :class:`Claims`.
        """
        if self._verifier is None:
            doc = await self._discovery.discover()
            config = JwtVerifierConfig(
                issuer=self._issuer_url,
                jwks_uri=doc["jwks_uri"],
                audience=self._audience,
            )
            self._verifier = JwtVerifier(config, self._http_client)

        return await self._verifier.verify(token)
