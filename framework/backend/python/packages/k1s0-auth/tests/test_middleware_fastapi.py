"""Tests for FastAPI auth middleware."""

from __future__ import annotations

from typing import Any
from unittest.mock import AsyncMock

import pytest

from k1s0_auth.errors import TokenInvalidError
from k1s0_auth.jwt.claims import Claims
from k1s0_auth.jwt.verifier import JwtVerifier
from k1s0_auth.middleware.fastapi import AuthMiddleware


async def _simple_app(scope: dict[str, Any], receive: Any, send: Any) -> None:
    """Minimal ASGI app that returns 200."""
    from starlette.responses import JSONResponse

    claims = scope.get("state", {}).get("claims")
    body = {"sub": claims.sub} if claims else {}
    response = JSONResponse(body)
    await response(scope, receive, send)


def _make_scope(
    path: str = "/api/test",
    auth_header: str | None = None,
) -> dict[str, Any]:
    """Create a minimal ASGI HTTP scope."""
    headers: list[tuple[bytes, bytes]] = []
    if auth_header:
        headers.append((b"authorization", auth_header.encode()))
    return {
        "type": "http",
        "path": path,
        "headers": headers,
        "state": {},
    }


class TestAuthMiddleware:
    """Test suite for AuthMiddleware."""

    @pytest.mark.asyncio()
    async def test_skip_path(self) -> None:
        verifier = AsyncMock(spec=JwtVerifier)
        middleware = AuthMiddleware(_simple_app, verifier)

        scope = _make_scope(path="/healthz")
        receive = AsyncMock()
        send = AsyncMock()
        await middleware(scope, receive, send)
        verifier.verify.assert_not_called()

    @pytest.mark.asyncio()
    async def test_missing_token_returns_401(self) -> None:
        verifier = AsyncMock(spec=JwtVerifier)
        middleware = AuthMiddleware(_simple_app, verifier)

        scope = _make_scope()
        receive = AsyncMock()
        responses: list[dict[str, Any]] = []

        async def capture_send(message: dict[str, Any]) -> None:
            responses.append(message)

        await middleware(scope, receive, capture_send)
        assert responses[0]["status"] == 401

    @pytest.mark.asyncio()
    async def test_valid_token_passes(self) -> None:
        verifier = AsyncMock(spec=JwtVerifier)
        verifier.verify.return_value = Claims(sub="user-1", roles=["admin"])
        middleware = AuthMiddleware(_simple_app, verifier)

        scope = _make_scope(auth_header="Bearer valid-token")
        receive = AsyncMock()
        responses: list[dict[str, Any]] = []

        async def capture_send(message: dict[str, Any]) -> None:
            responses.append(message)

        await middleware(scope, receive, capture_send)
        assert responses[0]["status"] == 200

    @pytest.mark.asyncio()
    async def test_invalid_token_returns_401(self) -> None:
        verifier = AsyncMock(spec=JwtVerifier)
        verifier.verify.side_effect = TokenInvalidError("bad token")
        middleware = AuthMiddleware(_simple_app, verifier)

        scope = _make_scope(auth_header="Bearer bad-token")
        receive = AsyncMock()
        responses: list[dict[str, Any]] = []

        async def capture_send(message: dict[str, Any]) -> None:
            responses.append(message)

        await middleware(scope, receive, capture_send)
        assert responses[0]["status"] == 401
