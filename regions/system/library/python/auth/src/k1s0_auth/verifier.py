"""JWT 検証"""

from __future__ import annotations

from typing import Any, cast

import jwt
from cryptography.hazmat.primitives.asymmetric.rsa import RSAPublicKey
from jwt.algorithms import RSAAlgorithm

from .exceptions import AuthError, AuthErrorCodes
from .jwks import JwksFetcher
from .models import TokenClaims

_STANDARD_CLAIMS = {"sub", "iss", "aud", "exp", "iat", "scope", "roles"}


def _parse_claims(payload: dict[str, Any]) -> TokenClaims:
    """JWT ペイロードを TokenClaims に変換する。"""
    aud = payload.get("aud", [])
    if isinstance(aud, str):
        aud = [aud]
    roles: list[str] = payload.get("roles", []) or payload.get("realm_access", {}).get("roles", [])
    return TokenClaims(
        sub=payload.get("sub", ""),
        iss=payload.get("iss", ""),
        aud=aud,
        exp=payload.get("exp", 0),
        iat=payload.get("iat", 0),
        scope=payload.get("scope", ""),
        roles=roles,
        extra={k: v for k, v in payload.items() if k not in _STANDARD_CLAIMS},
    )


class JwksVerifier:
    """JWKS を使用した JWT 検証クラス。"""

    def __init__(
        self,
        issuer: str,
        audience: str,
        fetcher: JwksFetcher,
    ) -> None:
        self._issuer = issuer
        self._audience = audience
        self._fetcher = fetcher

    def _get_public_key(self, keys: list[dict[str, Any]], kid: str | None) -> RSAPublicKey:
        """JWKS から公開鍵を取得する。"""
        if not keys:
            raise AuthError(
                code=AuthErrorCodes.JWKS_FETCH_ERROR,
                message="No keys found in JWKS",
            )
        key_data = keys[0]
        if kid:
            matched = [k for k in keys if k.get("kid") == kid]
            if matched:
                key_data = matched[0]
        return cast(RSAPublicKey, RSAAlgorithm.from_jwk(key_data))

    def verify_token(self, token: str) -> TokenClaims:
        """JWT トークンを同期検証する。"""
        try:
            header = jwt.get_unverified_header(token)
            kid = header.get("kid")
            keys = self._fetcher.fetch_keys()
            public_key = self._get_public_key(keys, kid)
            payload: dict[str, Any] = jwt.decode(
                token,
                public_key,
                algorithms=["RS256"],
                issuer=self._issuer,
                audience=self._audience,
            )
            return _parse_claims(payload)
        except jwt.ExpiredSignatureError as e:
            raise AuthError(
                code=AuthErrorCodes.EXPIRED_TOKEN, message="Token has expired", cause=e
            ) from e
        except jwt.InvalidTokenError as e:
            raise AuthError(
                code=AuthErrorCodes.INVALID_TOKEN, message=f"Invalid token: {e}", cause=e
            ) from e

    async def verify_token_async(self, token: str) -> TokenClaims:
        """JWT トークンを非同期検証する。"""
        try:
            header = jwt.get_unverified_header(token)
            kid = header.get("kid")
            keys = await self._fetcher.fetch_keys_async()
            public_key = self._get_public_key(keys, kid)
            payload: dict[str, Any] = jwt.decode(
                token,
                public_key,
                algorithms=["RS256"],
                issuer=self._issuer,
                audience=self._audience,
            )
            return _parse_claims(payload)
        except jwt.ExpiredSignatureError as e:
            raise AuthError(
                code=AuthErrorCodes.EXPIRED_TOKEN, message="Token has expired", cause=e
            ) from e
        except jwt.InvalidTokenError as e:
            raise AuthError(
                code=AuthErrorCodes.INVALID_TOKEN, message=f"Invalid token: {e}", cause=e
            ) from e
