"""service_auth モデルのユニットテスト"""

import time

import pytest
from k1s0_service_auth.models import ServiceToken, SpiffeId


def test_service_token_not_expired() -> None:
    """有効期限内のトークンは is_expired() が False を返すこと。"""
    token = ServiceToken(
        access_token="tok",
        token_type="Bearer",
        expires_at=time.time() + 3600,
    )
    assert token.is_expired() is False


def test_service_token_expired() -> None:
    """期限切れトークンは is_expired() が True を返すこと。"""
    token = ServiceToken(
        access_token="tok",
        token_type="Bearer",
        expires_at=time.time() - 1,
    )
    assert token.is_expired() is True


def test_service_token_from_response() -> None:
    """OAuth2 レスポンスから ServiceToken を生成できること。"""
    response = {
        "access_token": "my_token",
        "token_type": "Bearer",
        "expires_in": 3600,
        "scope": "read write",
    }
    token = ServiceToken.from_response(response)
    assert token.access_token == "my_token"
    assert token.scope == "read write"
    assert not token.is_expired()


def test_spiffe_id_parse_valid() -> None:
    """有効な SPIFFE URI をパースできること。"""
    uri = "spiffe://k1s0.local/ns/system/sa/auth-service"
    spiffe = SpiffeId.parse(uri)
    assert spiffe.trust_domain == "k1s0.local"
    assert spiffe.namespace == "system"
    assert spiffe.service_account == "auth-service"


def test_spiffe_id_to_uri() -> None:
    """SpiffeId を URI に変換できること。"""
    spiffe = SpiffeId(trust_domain="k1s0.local", namespace="system", service_account="auth")
    assert spiffe.to_uri() == "spiffe://k1s0.local/ns/system/sa/auth"


def test_spiffe_id_round_trip() -> None:
    """URI → SpiffeId → URI のラウンドトリップ確認。"""
    uri = "spiffe://example.com/ns/default/sa/myservice"
    assert SpiffeId.parse(uri).to_uri() == uri


def test_spiffe_id_invalid_format() -> None:
    """不正な形式で ValueError が発生すること。"""
    with pytest.raises(ValueError, match="Invalid SPIFFE ID"):
        SpiffeId.parse("not-a-spiffe-uri")

    with pytest.raises(ValueError):
        SpiffeId.parse("spiffe://missing-parts")
