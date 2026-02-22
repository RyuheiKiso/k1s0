"""HttpJwksFetcher のユニットテスト（respx によるHTTPモック）"""

from __future__ import annotations

import time

import httpx
import pytest
import respx
from k1s0_auth.exceptions import AuthError, AuthErrorCodes
from k1s0_auth.jwks import HttpJwksFetcher

JWKS_URI = "https://auth.example.com/.well-known/jwks.json"
SAMPLE_KEYS = [{"kty": "RSA", "kid": "key-1", "n": "abc", "e": "AQAB"}]


@respx.mock
def test_fetch_keys_success() -> None:
    """正常系: JWKS キーを HTTP で取得できること。"""
    respx.get(JWKS_URI).mock(return_value=httpx.Response(200, json={"keys": SAMPLE_KEYS}))
    fetcher = HttpJwksFetcher(jwks_uri=JWKS_URI)
    keys = fetcher.fetch_keys()
    assert keys == SAMPLE_KEYS


@respx.mock
def test_fetch_keys_uses_cache() -> None:
    """キャッシュが有効な間は HTTP リクエストを送らないこと。"""
    respx.get(JWKS_URI).mock(return_value=httpx.Response(200, json={"keys": SAMPLE_KEYS}))
    fetcher = HttpJwksFetcher(jwks_uri=JWKS_URI, ttl_seconds=300)
    # 最初の呼び出し
    keys1 = fetcher.fetch_keys()
    # 2回目はキャッシュから
    keys2 = fetcher.fetch_keys()
    assert keys1 == keys2
    # HTTP リクエストは1回だけ
    assert respx.calls.call_count == 1


@respx.mock
def test_fetch_keys_http_error() -> None:
    """HTTP エラー時に AuthError(JWKS_FETCH_ERROR) が発生すること。"""
    respx.get(JWKS_URI).mock(return_value=httpx.Response(500, text="Internal Server Error"))
    fetcher = HttpJwksFetcher(jwks_uri=JWKS_URI)
    with pytest.raises(AuthError) as exc_info:
        fetcher.fetch_keys()
    assert exc_info.value.code == AuthErrorCodes.JWKS_FETCH_ERROR


@respx.mock
def test_fetch_keys_network_error() -> None:
    """ネットワークエラー時に AuthError(JWKS_FETCH_ERROR) が発生すること。"""
    respx.get(JWKS_URI).mock(side_effect=httpx.ConnectError("connection failed"))
    fetcher = HttpJwksFetcher(jwks_uri=JWKS_URI)
    with pytest.raises(AuthError) as exc_info:
        fetcher.fetch_keys()
    assert exc_info.value.code == AuthErrorCodes.JWKS_FETCH_ERROR


@respx.mock
async def test_fetch_keys_async_success() -> None:
    """非同期正常系: JWKS キーを取得できること。"""
    respx.get(JWKS_URI).mock(return_value=httpx.Response(200, json={"keys": SAMPLE_KEYS}))
    fetcher = HttpJwksFetcher(jwks_uri=JWKS_URI)
    keys = await fetcher.fetch_keys_async()
    assert keys == SAMPLE_KEYS


@respx.mock
async def test_fetch_keys_async_uses_cache() -> None:
    """非同期でキャッシュが有効な間は HTTP リクエストを送らないこと。"""
    respx.get(JWKS_URI).mock(return_value=httpx.Response(200, json={"keys": SAMPLE_KEYS}))
    fetcher = HttpJwksFetcher(jwks_uri=JWKS_URI, ttl_seconds=300)
    await fetcher.fetch_keys_async()
    keys2 = await fetcher.fetch_keys_async()
    assert keys2 == SAMPLE_KEYS
    assert respx.calls.call_count == 1


@respx.mock
async def test_fetch_keys_async_http_error() -> None:
    """非同期 HTTP エラー時に AuthError(JWKS_FETCH_ERROR) が発生すること。"""
    respx.get(JWKS_URI).mock(return_value=httpx.Response(503, text="Service Unavailable"))
    fetcher = HttpJwksFetcher(jwks_uri=JWKS_URI)
    with pytest.raises(AuthError) as exc_info:
        await fetcher.fetch_keys_async()
    assert exc_info.value.code == AuthErrorCodes.JWKS_FETCH_ERROR


@respx.mock
async def test_fetch_keys_async_network_error() -> None:
    """非同期ネットワークエラー時に AuthError(JWKS_FETCH_ERROR) が発生すること。"""
    respx.get(JWKS_URI).mock(side_effect=httpx.ConnectError("connection failed"))
    fetcher = HttpJwksFetcher(jwks_uri=JWKS_URI)
    with pytest.raises(AuthError) as exc_info:
        await fetcher.fetch_keys_async()
    assert exc_info.value.code == AuthErrorCodes.JWKS_FETCH_ERROR


def test_cache_expires_after_ttl() -> None:
    """TTL が切れるとキャッシュが無効になること。"""
    fetcher = HttpJwksFetcher(jwks_uri=JWKS_URI, ttl_seconds=1)
    # 手動でキャッシュを設定（有効状態）
    fetcher._cached_keys = SAMPLE_KEYS
    fetcher._cache_expires_at = time.time() + 1000
    assert fetcher._is_cache_valid() is True
    # 有効期限を過去に設定
    fetcher._cache_expires_at = time.time() - 1
    assert fetcher._is_cache_valid() is False


def test_cache_invalid_when_empty() -> None:
    """キーが空の場合はキャッシュ無効と判定されること。"""
    fetcher = HttpJwksFetcher(jwks_uri=JWKS_URI)
    fetcher._cached_keys = []
    fetcher._cache_expires_at = time.time() + 1000
    assert fetcher._is_cache_valid() is False
