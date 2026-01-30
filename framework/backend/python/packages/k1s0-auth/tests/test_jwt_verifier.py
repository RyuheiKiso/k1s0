"""Tests for JwtVerifier."""

from __future__ import annotations

from typing import Any
from unittest.mock import AsyncMock, patch

import httpx
import pytest

from k1s0_auth.errors import TokenExpiredError, TokenInvalidError
from k1s0_auth.jwt.config import JwtVerifierConfig
from k1s0_auth.jwt.verifier import JwtVerifier


@pytest.fixture()
def config() -> JwtVerifierConfig:
    return JwtVerifierConfig(
        issuer="https://auth.example.com",
        jwks_uri="https://auth.example.com/.well-known/jwks.json",
        audience="my-api",
    )


@pytest.fixture()
def mock_http_client(jwks_response: dict[str, Any]) -> httpx.AsyncClient:
    """Create a mock httpx client that returns the JWKS."""
    client = AsyncMock(spec=httpx.AsyncClient)
    response = AsyncMock(spec=httpx.Response)
    response.json.return_value = jwks_response
    response.raise_for_status = lambda: None
    client.get = AsyncMock(return_value=response)
    return client


class TestJwtVerifier:
    """Test suite for JwtVerifier."""

    @pytest.mark.asyncio()
    async def test_verify_valid_token(
        self,
        config: JwtVerifierConfig,
        mock_http_client: httpx.AsyncClient,
        make_token: Any,
    ) -> None:
        verifier = JwtVerifier(config, mock_http_client)
        token = make_token(sub="user-42", roles=["admin"])
        claims = await verifier.verify(token)
        assert claims.sub == "user-42"
        assert claims.has_role("admin")

    @pytest.mark.asyncio()
    async def test_verify_expired_token(
        self,
        config: JwtVerifierConfig,
        mock_http_client: httpx.AsyncClient,
        make_token: Any,
    ) -> None:
        verifier = JwtVerifier(config, mock_http_client)
        token = make_token(exp_offset=-3600)
        with pytest.raises(TokenExpiredError):
            await verifier.verify(token)

    @pytest.mark.asyncio()
    async def test_verify_invalid_token(
        self,
        config: JwtVerifierConfig,
        mock_http_client: httpx.AsyncClient,
    ) -> None:
        verifier = JwtVerifier(config, mock_http_client)
        with pytest.raises(TokenInvalidError):
            await verifier.verify("not.a.token")

    @pytest.mark.asyncio()
    async def test_verify_wrong_audience(
        self,
        mock_http_client: httpx.AsyncClient,
        make_token: Any,
    ) -> None:
        config = JwtVerifierConfig(
            issuer="https://auth.example.com",
            jwks_uri="https://auth.example.com/.well-known/jwks.json",
            audience="wrong-api",
        )
        verifier = JwtVerifier(config, mock_http_client)
        token = make_token(audience="my-api")
        with pytest.raises(TokenInvalidError):
            await verifier.verify(token)

    @pytest.mark.asyncio()
    async def test_jwks_caching(
        self,
        config: JwtVerifierConfig,
        mock_http_client: httpx.AsyncClient,
        make_token: Any,
    ) -> None:
        verifier = JwtVerifier(config, mock_http_client)
        token1 = make_token(sub="u1")
        token2 = make_token(sub="u2")
        await verifier.verify(token1)
        await verifier.verify(token2)
        # JWKS should only be fetched once due to caching
        assert mock_http_client.get.call_count == 1  # type: ignore[union-attr]
