"""HttpServiceAuthClient のユニットテスト（respx モック）"""

import time

import httpx
import pytest
import respx
from k1s0_service_auth.exceptions import ServiceAuthError, ServiceAuthErrorCodes
from k1s0_service_auth.http_client import HttpServiceAuthClient
from k1s0_service_auth.models import ServiceAuthConfig

TOKEN_URL = "https://auth.example.com/token"


def make_client() -> HttpServiceAuthClient:
    return HttpServiceAuthClient(
        ServiceAuthConfig(
            token_url=TOKEN_URL,
            client_id="my-client",
            client_secret="my-secret",
            scope="api",
        )
    )


@respx.mock
def test_get_token_success() -> None:
    """トークン取得成功。"""
    respx.post(TOKEN_URL).mock(
        return_value=httpx.Response(
            200,
            json={"access_token": "tok123", "token_type": "Bearer", "expires_in": 3600},
        )
    )
    client = make_client()
    token = client.get_token()
    assert token.access_token == "tok123"
    assert not token.is_expired()


@respx.mock
def test_get_token_failure() -> None:
    """トークン取得失敗時に ServiceAuthError が発生すること。"""
    respx.post(TOKEN_URL).mock(return_value=httpx.Response(401, text="Unauthorized"))
    client = make_client()
    with pytest.raises(ServiceAuthError) as exc_info:
        client.get_token()
    assert exc_info.value.code == ServiceAuthErrorCodes.TOKEN_REQUEST_FAILED


@respx.mock
def test_get_cached_token_uses_cache() -> None:
    """2回目の呼び出しではキャッシュが使用されること。"""
    call_count = 0

    def handler(request):
        nonlocal call_count
        call_count += 1
        return httpx.Response(
            200,
            json={"access_token": f"tok{call_count}", "token_type": "Bearer", "expires_in": 3600},
        )

    respx.post(TOKEN_URL).mock(side_effect=handler)
    client = make_client()
    token1 = client.get_cached_token()
    token2 = client.get_cached_token()
    assert token1.access_token == token2.access_token
    assert call_count == 1  # 1回しか呼ばれないこと


@respx.mock
async def test_get_token_async_success() -> None:
    """非同期トークン取得成功。"""
    respx.post(TOKEN_URL).mock(
        return_value=httpx.Response(
            200,
            json={"access_token": "async_tok", "token_type": "Bearer", "expires_in": 1800},
        )
    )
    client = make_client()
    token = await client.get_token_async()
    assert token.access_token == "async_tok"


@respx.mock
def test_get_cached_token_refreshes_expired() -> None:
    """期限切れトークンは自動更新されること。"""
    respx.post(TOKEN_URL).mock(
        return_value=httpx.Response(
            200,
            json={"access_token": "new_tok", "token_type": "Bearer", "expires_in": 3600},
        )
    )
    client = make_client()
    # 期限切れトークンを強制セット
    from k1s0_service_auth.models import ServiceToken

    client._cached_token = ServiceToken(
        access_token="old_tok",
        token_type="Bearer",
        expires_at=time.time() - 100,
    )
    token = client.get_cached_token()
    assert token.access_token == "new_tok"
