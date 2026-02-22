"""JwksVerifier のユニットテスト（モック使用）"""

from unittest.mock import AsyncMock, MagicMock

import pytest
from k1s0_auth.exceptions import AuthError, AuthErrorCodes
from k1s0_auth.jwks import JwksFetcher
from k1s0_auth.verifier import JwksVerifier, _parse_claims


def test_parse_claims_basic() -> None:
    """基本的なクレームのパース。"""
    payload = {
        "sub": "user-123",
        "iss": "https://issuer.example.com",
        "aud": "api",
        "exp": 9999999999,
        "iat": 0,
        "scope": "read write",
    }
    claims = _parse_claims(payload)
    assert claims.sub == "user-123"
    assert claims.aud == ["api"]
    assert claims.has_scope("read") is True
    assert claims.has_scope("delete") is False


def test_parse_claims_with_roles() -> None:
    """ロール付きクレームのパース。"""
    payload = {
        "sub": "user-123",
        "iss": "https://iss",
        "aud": ["api"],
        "exp": 9999999999,
        "iat": 0,
        "roles": ["admin"],
    }
    claims = _parse_claims(payload)
    assert claims.has_role("admin") is True
    assert claims.has_role("viewer") is False


def test_verifier_no_keys_raises() -> None:
    """JWKS キーが空の場合に AuthError が発生すること。"""
    mock_fetcher = MagicMock(spec=JwksFetcher)
    mock_fetcher.fetch_keys.return_value = []
    verifier = JwksVerifier(issuer="https://iss", audience="api", fetcher=mock_fetcher)

    with pytest.raises(AuthError) as exc_info:
        verifier.verify_token("fake.token.here")
    assert exc_info.value.code in (AuthErrorCodes.JWKS_FETCH_ERROR, AuthErrorCodes.INVALID_TOKEN)


async def test_verifier_async_no_keys_raises() -> None:
    """非同期検証で JWKS キーが空の場合に AuthError が発生すること。"""
    mock_fetcher = MagicMock(spec=JwksFetcher)
    mock_fetcher.fetch_keys_async = AsyncMock(return_value=[])
    verifier = JwksVerifier(issuer="https://iss", audience="api", fetcher=mock_fetcher)

    with pytest.raises(AuthError):
        await verifier.verify_token_async("fake.token.here")
