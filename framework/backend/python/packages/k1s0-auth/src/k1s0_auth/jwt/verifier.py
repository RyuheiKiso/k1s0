"""JWT token verification using JWKS."""

from __future__ import annotations

import time
from typing import Any

import httpx
import jwt
from jwt import PyJWK

from k1s0_auth.errors import TokenExpiredError, TokenInvalidError
from k1s0_auth.jwt.claims import Claims
from k1s0_auth.jwt.config import JwtVerifierConfig

_JWKS_CACHE_TTL = 300  # seconds


class JwtVerifier:
    """Verifies JWT tokens against a remote JWKS endpoint.

    Args:
        config: Verifier configuration.
        http_client: Optional shared ``httpx.AsyncClient``.
    """

    def __init__(
        self,
        config: JwtVerifierConfig,
        http_client: httpx.AsyncClient | None = None,
    ) -> None:
        self._config = config
        self._http_client = http_client
        self._owns_client = http_client is None
        self._jwks_cache: dict[str, Any] = {}
        self._jwks_last_fetched: float | None = None

    async def verify(self, token: str) -> Claims:
        """Decode and verify a JWT token.

        Args:
            token: The raw JWT string.

        Returns:
            Parsed :class:`Claims` from the token.

        Raises:
            TokenExpiredError: If the token has expired.
            TokenInvalidError: If the token is malformed or the signature is invalid.
        """
        try:
            unverified_header = jwt.get_unverified_header(token)
        except jwt.exceptions.DecodeError as exc:
            raise TokenInvalidError(f"Cannot decode token header: {exc}") from exc

        signing_key = await self._get_signing_key(unverified_header)

        try:
            payload: dict[str, Any] = jwt.decode(
                token,
                signing_key.key,
                algorithms=self._config.algorithms,
                audience=self._config.audience,
                issuer=self._config.issuer,
                leeway=self._config.clock_skew,
            )
        except jwt.ExpiredSignatureError as exc:
            raise TokenExpiredError() from exc
        except jwt.InvalidTokenError as exc:
            raise TokenInvalidError(str(exc)) from exc

        return Claims(
            sub=payload.get("sub", ""),
            roles=payload.get("roles", []),
            permissions=payload.get("permissions", []),
            groups=payload.get("groups", []),
            tenant_id=payload.get("tenant_id"),
            custom={
                k: v
                for k, v in payload.items()
                if k
                not in {
                    "sub",
                    "roles",
                    "permissions",
                    "groups",
                    "tenant_id",
                    "iss",
                    "aud",
                    "exp",
                    "iat",
                    "nbf",
                    "jti",
                }
            },
        )

    async def _fetch_jwks(self) -> dict[str, Any]:
        """Fetch the JWKS from the configured URI, using cache when valid."""
        now = time.monotonic()
        if (
            self._jwks_cache
            and self._jwks_last_fetched is not None
            and (now - self._jwks_last_fetched) < _JWKS_CACHE_TTL
        ):
            return self._jwks_cache

        client = self._http_client or httpx.AsyncClient()
        try:
            response = await client.get(self._config.jwks_uri)
            response.raise_for_status()
            self._jwks_cache = response.json()
            self._jwks_last_fetched = now
            return self._jwks_cache
        except httpx.HTTPError as exc:
            raise TokenInvalidError(f"Failed to fetch JWKS: {exc}") from exc
        finally:
            if self._owns_client and client is not self._http_client:
                await client.aclose()

    async def _get_signing_key(self, token_header: dict[str, Any]) -> PyJWK:
        """Find the signing key matching the token's ``kid`` header."""
        jwks_data = await self._fetch_jwks()
        kid = token_header.get("kid")

        keys: list[dict[str, Any]] = jwks_data.get("keys", [])
        for key_data in keys:
            if kid is not None and key_data.get("kid") != kid:
                continue
            return PyJWK(key_data)

        raise TokenInvalidError(f"No matching key found for kid={kid}")
