"""FastAPI authentication middleware and dependency."""

from __future__ import annotations

import json
from typing import Any

from starlette.requests import Request
from starlette.responses import Response
from starlette.types import ASGIApp, Receive, Scope, Send

from k1s0_auth.errors import AuthError
from k1s0_auth.jwt.claims import Claims
from k1s0_auth.jwt.verifier import JwtVerifier


class AuthMiddleware:
    """ASGI middleware that validates Bearer tokens on incoming requests.

    Requests to paths in ``skip_paths`` bypass authentication.
    On success, ``request.state.claims`` is set to the parsed :class:`Claims`.

    Args:
        app: The ASGI application to wrap.
        verifier: The JWT verifier instance.
        skip_paths: Paths that do not require authentication.
    """

    def __init__(
        self,
        app: ASGIApp,
        verifier: JwtVerifier,
        skip_paths: list[str] | None = None,
    ) -> None:
        self._app = app
        self._verifier = verifier
        self._skip_paths = skip_paths or ["/healthz", "/readyz"]

    async def __call__(self, scope: Scope, receive: Receive, send: Send) -> None:
        """Process the ASGI request."""
        if scope["type"] != "http":
            await self._app(scope, receive, send)
            return

        path: str = scope.get("path", "")
        if path in self._skip_paths:
            await self._app(scope, receive, send)
            return

        headers = dict(scope.get("headers", []))
        auth_header = headers.get(b"authorization", b"").decode()

        if not auth_header.startswith("Bearer "):
            response = self._error_response(401, "auth.missing_token", "Missing Bearer token")
            await response(scope, receive, send)
            return

        token = auth_header[7:]
        try:
            claims = await self._verifier.verify(token)
        except AuthError as exc:
            response = self._error_response(401, exc.error_code, str(exc))
            await response(scope, receive, send)
            return

        scope.setdefault("state", {})
        scope["state"]["claims"] = claims

        await self._app(scope, receive, send)

    @staticmethod
    def _error_response(status: int, error_code: str, detail: str) -> Response:
        """Create a JSON error response."""
        body: dict[str, Any] = {
            "status": status,
            "title": "Unauthorized",
            "detail": detail,
            "error_code": error_code,
        }
        return Response(
            content=json.dumps(body),
            status_code=status,
            media_type="application/json",
        )


async def require_auth(request: Request) -> Claims:
    """FastAPI dependency that extracts authenticated claims from the request.

    Usage::

        @app.get("/protected")
        async def protected(claims: Claims = Depends(require_auth)):
            return {"sub": claims.sub}

    Raises:
        HTTPException: If no claims are present on the request state.
    """
    claims: Claims | None = getattr(request.state, "claims", None)
    if claims is None:
        from starlette.exceptions import HTTPException

        raise HTTPException(status_code=401, detail="Not authenticated")
    return claims
