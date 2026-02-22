"""OAuth2 Client Credentials フロー実装"""

from __future__ import annotations

from typing import Any

import httpx

from .client import ServiceAuthClient
from .exceptions import ServiceAuthError, ServiceAuthErrorCodes
from .models import ServiceAuthConfig, ServiceToken


class HttpServiceAuthClient(ServiceAuthClient):
    """httpx を使った OAuth2 Client Credentials フロー実装。"""

    def __init__(self, config: ServiceAuthConfig) -> None:
        self._config = config
        self._cached_token: ServiceToken | None = None

    def _request_token(self) -> dict[str, Any]:
        data = {
            "grant_type": "client_credentials",
            "client_id": self._config.client_id,
            "client_secret": self._config.client_secret,
        }
        if self._config.scope:
            data["scope"] = self._config.scope
        try:
            with httpx.Client(timeout=self._config.timeout_seconds) as client:
                resp = client.post(self._config.token_url, data=data)
            resp.raise_for_status()
            result: dict[str, Any] = resp.json()
            return result
        except httpx.HTTPStatusError as e:
            raise ServiceAuthError(
                code=ServiceAuthErrorCodes.TOKEN_REQUEST_FAILED,
                message=f"Token request failed: HTTP {e.response.status_code}",
                cause=e,
            ) from e
        except httpx.HTTPError as e:
            raise ServiceAuthError(
                code=ServiceAuthErrorCodes.TOKEN_REQUEST_FAILED,
                message=f"Token request failed: {e}",
                cause=e,
            ) from e

    async def _request_token_async(self) -> dict[str, Any]:
        data = {
            "grant_type": "client_credentials",
            "client_id": self._config.client_id,
            "client_secret": self._config.client_secret,
        }
        if self._config.scope:
            data["scope"] = self._config.scope
        try:
            async with httpx.AsyncClient(timeout=self._config.timeout_seconds) as client:
                resp = await client.post(self._config.token_url, data=data)
            resp.raise_for_status()
            result: dict[str, Any] = resp.json()
            return result
        except httpx.HTTPStatusError as e:
            raise ServiceAuthError(
                code=ServiceAuthErrorCodes.TOKEN_REQUEST_FAILED,
                message=f"Token request failed async: HTTP {e.response.status_code}",
                cause=e,
            ) from e
        except httpx.HTTPError as e:
            raise ServiceAuthError(
                code=ServiceAuthErrorCodes.TOKEN_REQUEST_FAILED,
                message=f"Token request failed async: {e}",
                cause=e,
            ) from e

    def get_token(self) -> ServiceToken:
        """新しいアクセストークンを取得する。"""
        response = self._request_token()
        return ServiceToken.from_response(response)

    async def get_token_async(self) -> ServiceToken:
        """非同期で新しいアクセストークンを取得する。"""
        response = await self._request_token_async()
        return ServiceToken.from_response(response)

    def get_cached_token(self) -> ServiceToken:
        """キャッシュされたトークンを返す（期限切れの場合は更新）。"""
        if self._cached_token is None or self._cached_token.is_expired():
            self._cached_token = self.get_token()
        return self._cached_token

    async def get_cached_token_async(self) -> ServiceToken:
        """非同期でキャッシュされたトークンを返す（期限切れの場合は更新）。"""
        if self._cached_token is None or self._cached_token.is_expired():
            self._cached_token = await self.get_token_async()
        return self._cached_token
