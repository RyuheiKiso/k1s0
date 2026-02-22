"""JWKS フェッチャー"""

from __future__ import annotations

import time
from abc import ABC, abstractmethod
from typing import Any

import httpx

from .exceptions import AuthError, AuthErrorCodes


class JwksFetcher(ABC):
    """JWKS フェッチャー抽象基底クラス。"""

    @abstractmethod
    def fetch_keys(self) -> list[dict[str, Any]]:
        """JWKS キーリストを取得する。"""
        ...

    @abstractmethod
    async def fetch_keys_async(self) -> list[dict[str, Any]]:
        """非同期で JWKS キーリストを取得する。"""
        ...


class HttpJwksFetcher(JwksFetcher):
    """HTTP で JWKS を取得するフェッチャー。TTL キャッシュ付き。"""

    def __init__(self, jwks_uri: str, ttl_seconds: int = 300) -> None:
        self._jwks_uri = jwks_uri
        self._ttl_seconds = ttl_seconds
        self._cached_keys: list[dict[str, Any]] = []
        self._cache_expires_at: float = 0.0

    def _is_cache_valid(self) -> bool:
        return time.time() < self._cache_expires_at and bool(self._cached_keys)

    def fetch_keys(self) -> list[dict[str, Any]]:
        """同期で JWKS キーを取得する。"""
        if self._is_cache_valid():
            return self._cached_keys
        try:
            resp = httpx.get(self._jwks_uri, timeout=10.0)
            resp.raise_for_status()
            data: dict[str, Any] = resp.json()
            self._cached_keys = data.get("keys", [])
            self._cache_expires_at = time.time() + self._ttl_seconds
            return self._cached_keys
        except httpx.HTTPError as e:
            raise AuthError(
                code=AuthErrorCodes.JWKS_FETCH_ERROR,
                message=f"Failed to fetch JWKS from {self._jwks_uri}: {e}",
                cause=e,
            ) from e

    async def fetch_keys_async(self) -> list[dict[str, Any]]:
        """非同期で JWKS キーを取得する。"""
        if self._is_cache_valid():
            return self._cached_keys
        try:
            async with httpx.AsyncClient() as client:
                resp = await client.get(self._jwks_uri, timeout=10.0)
                resp.raise_for_status()
                data: dict[str, Any] = resp.json()
                self._cached_keys = data.get("keys", [])
                self._cache_expires_at = time.time() + self._ttl_seconds
                return self._cached_keys
        except httpx.HTTPError as e:
            raise AuthError(
                code=AuthErrorCodes.JWKS_FETCH_ERROR,
                message=f"Failed to fetch JWKS from {self._jwks_uri}: {e}",
                cause=e,
            ) from e
