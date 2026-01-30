"""Tests for OidcDiscovery."""

from __future__ import annotations

from unittest.mock import AsyncMock

import httpx
import pytest

from k1s0_auth.errors import DiscoveryError
from k1s0_auth.oidc.discovery import OidcDiscovery


@pytest.fixture()
def discovery_response() -> dict[str, str]:
    return {
        "issuer": "https://auth.example.com",
        "jwks_uri": "https://auth.example.com/.well-known/jwks.json",
        "authorization_endpoint": "https://auth.example.com/authorize",
        "token_endpoint": "https://auth.example.com/token",
        "userinfo_endpoint": "https://auth.example.com/userinfo",
    }


class TestOidcDiscovery:
    """Test suite for OidcDiscovery."""

    @pytest.mark.asyncio()
    async def test_discover_success(self, discovery_response: dict[str, str]) -> None:
        client = AsyncMock(spec=httpx.AsyncClient)
        response = AsyncMock(spec=httpx.Response)
        response.json.return_value = discovery_response
        response.raise_for_status = lambda: None
        client.get = AsyncMock(return_value=response)

        discovery = OidcDiscovery("https://auth.example.com", client)
        doc = await discovery.discover()
        assert doc["jwks_uri"] == "https://auth.example.com/.well-known/jwks.json"

    @pytest.mark.asyncio()
    async def test_discover_caches(self, discovery_response: dict[str, str]) -> None:
        client = AsyncMock(spec=httpx.AsyncClient)
        response = AsyncMock(spec=httpx.Response)
        response.json.return_value = discovery_response
        response.raise_for_status = lambda: None
        client.get = AsyncMock(return_value=response)

        discovery = OidcDiscovery("https://auth.example.com", client)
        await discovery.discover()
        await discovery.discover()
        assert client.get.call_count == 1

    @pytest.mark.asyncio()
    async def test_discover_failure(self) -> None:
        client = AsyncMock(spec=httpx.AsyncClient)
        client.get = AsyncMock(side_effect=httpx.HTTPError("connection failed"))

        discovery = OidcDiscovery("https://auth.example.com", client)
        with pytest.raises(DiscoveryError):
            await discovery.discover()
