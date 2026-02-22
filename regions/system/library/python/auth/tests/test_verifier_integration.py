"""JwksVerifier の統合テスト（実際の RSA 鍵ペア使用）"""

from __future__ import annotations

import time
from unittest.mock import AsyncMock, MagicMock

import jwt
import pytest
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric import rsa
from k1s0_auth.exceptions import AuthError, AuthErrorCodes
from k1s0_auth.jwks import JwksFetcher
from k1s0_auth.verifier import JwksVerifier, _parse_claims


def generate_rsa_key_pair():
    """テスト用 RSA 鍵ペアを生成する。"""
    private_key = rsa.generate_private_key(
        public_exponent=65537,
        key_size=2048,
    )
    return private_key, private_key.public_key()


def make_jwk_from_private_key(private_key, kid: str = "test-key-1") -> dict:
    """RSA 秘密鍵から JWK 形式の公開鍵辞書を生成する。"""
    import json

    from jwt.algorithms import RSAAlgorithm

    public_key = private_key.public_key()
    jwk_str = RSAAlgorithm.to_jwk(public_key)
    jwk = json.loads(jwk_str)
    jwk["kid"] = kid
    return jwk


def make_token(
    private_key,
    kid: str = "test-key-1",
    sub: str = "user-123",
    iss: str = "https://issuer.example.com",
    aud: str = "my-api",
    exp_offset: int = 3600,
    extra_claims: dict | None = None,
) -> str:
    """テスト用 JWT トークンを生成する。"""
    now = int(time.time())
    payload = {
        "sub": sub,
        "iss": iss,
        "aud": aud,
        "exp": now + exp_offset,
        "iat": now,
        "scope": "read write",
        "roles": ["user"],
    }
    if extra_claims:
        payload.update(extra_claims)

    private_pem = private_key.private_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PrivateFormat.TraditionalOpenSSL,
        encryption_algorithm=serialization.NoEncryption(),
    )
    return jwt.encode(payload, private_pem, algorithm="RS256", headers={"kid": kid})


ISSUER = "https://issuer.example.com"
AUDIENCE = "my-api"


@pytest.fixture
def rsa_key_pair():
    return generate_rsa_key_pair()


@pytest.fixture
def verifier_with_key(rsa_key_pair):
    """RSA 鍵ペアを使った JwksVerifier を返すフィクスチャ。"""
    private_key, _ = rsa_key_pair
    jwk = make_jwk_from_private_key(private_key, kid="test-key-1")

    mock_fetcher = MagicMock(spec=JwksFetcher)
    mock_fetcher.fetch_keys.return_value = [jwk]
    mock_fetcher.fetch_keys_async = AsyncMock(return_value=[jwk])

    return JwksVerifier(issuer=ISSUER, audience=AUDIENCE, fetcher=mock_fetcher), private_key


def test_verify_token_success(verifier_with_key) -> None:
    """正常なトークンの検証が成功すること。"""
    verifier, private_key = verifier_with_key
    token = make_token(private_key)
    claims = verifier.verify_token(token)
    assert claims.sub == "user-123"
    assert claims.iss == ISSUER
    assert AUDIENCE in claims.aud
    assert claims.has_scope("read") is True
    assert claims.has_role("user") is True


def test_verify_token_kid_match(verifier_with_key) -> None:
    """kid が一致する鍵で検証できること。"""
    verifier, private_key = verifier_with_key
    token = make_token(private_key, kid="test-key-1")
    claims = verifier.verify_token(token)
    assert claims.sub == "user-123"


def test_verify_token_kid_no_match_falls_back_to_first(rsa_key_pair) -> None:
    """kid が一致しない場合、最初の鍵にフォールバックすること。"""
    private_key, _ = rsa_key_pair
    jwk = make_jwk_from_private_key(private_key, kid="actual-key-id")

    mock_fetcher = MagicMock(spec=JwksFetcher)
    mock_fetcher.fetch_keys.return_value = [jwk]

    verifier = JwksVerifier(issuer=ISSUER, audience=AUDIENCE, fetcher=mock_fetcher)
    # kid が一致しないトークン（でも鍵は同一）→ 最初の鍵でフォールバック検証
    token = make_token(private_key, kid="other-kid")
    claims = verifier.verify_token(token)
    assert claims.sub == "user-123"


def test_verify_token_expired(verifier_with_key) -> None:
    """期限切れトークンで EXPIRED_TOKEN エラーが発生すること。"""
    verifier, private_key = verifier_with_key
    token = make_token(private_key, exp_offset=-3600)
    with pytest.raises(AuthError) as exc_info:
        verifier.verify_token(token)
    assert exc_info.value.code == AuthErrorCodes.EXPIRED_TOKEN


def test_verify_token_wrong_issuer(verifier_with_key) -> None:
    """発行者不一致で INVALID_TOKEN エラーが発生すること。"""
    verifier, private_key = verifier_with_key
    token = make_token(private_key, iss="https://wrong-issuer.com")
    with pytest.raises(AuthError) as exc_info:
        verifier.verify_token(token)
    assert exc_info.value.code == AuthErrorCodes.INVALID_TOKEN


def test_verify_token_wrong_audience(verifier_with_key) -> None:
    """対象者不一致で INVALID_TOKEN エラーが発生すること。"""
    verifier, private_key = verifier_with_key
    token = make_token(private_key, aud="wrong-audience")
    with pytest.raises(AuthError) as exc_info:
        verifier.verify_token(token)
    assert exc_info.value.code == AuthErrorCodes.INVALID_TOKEN


def test_verify_token_invalid_signature(rsa_key_pair) -> None:
    """署名が不正なトークンで INVALID_TOKEN エラーが発生すること。"""
    private_key, _ = rsa_key_pair
    # 別の鍵で署名されたトークン
    other_private_key, _ = generate_rsa_key_pair()
    token = make_token(other_private_key, kid="test-key-1")

    jwk = make_jwk_from_private_key(private_key, kid="test-key-1")
    mock_fetcher = MagicMock(spec=JwksFetcher)
    mock_fetcher.fetch_keys.return_value = [jwk]

    verifier = JwksVerifier(issuer=ISSUER, audience=AUDIENCE, fetcher=mock_fetcher)
    with pytest.raises(AuthError) as exc_info:
        verifier.verify_token(token)
    assert exc_info.value.code == AuthErrorCodes.INVALID_TOKEN


async def test_verify_token_async_success(verifier_with_key) -> None:
    """非同期検証の正常系。"""
    verifier, private_key = verifier_with_key
    token = make_token(private_key)
    claims = await verifier.verify_token_async(token)
    assert claims.sub == "user-123"
    assert claims.has_scope("write") is True


async def test_verify_token_async_expired(verifier_with_key) -> None:
    """非同期検証で期限切れトークンの EXPIRED_TOKEN エラーが発生すること。"""
    verifier, private_key = verifier_with_key
    token = make_token(private_key, exp_offset=-3600)
    with pytest.raises(AuthError) as exc_info:
        await verifier.verify_token_async(token)
    assert exc_info.value.code == AuthErrorCodes.EXPIRED_TOKEN


async def test_verify_token_async_invalid(verifier_with_key) -> None:
    """非同期検証で不正トークンの INVALID_TOKEN エラーが発生すること。"""
    verifier, private_key = verifier_with_key
    token = make_token(private_key, iss="https://wrong.com")
    with pytest.raises(AuthError) as exc_info:
        await verifier.verify_token_async(token)
    assert exc_info.value.code == AuthErrorCodes.INVALID_TOKEN


def test_parse_claims_realm_access_roles() -> None:
    """realm_access.roles からロールをパースできること。"""
    payload = {
        "sub": "user-456",
        "iss": "https://iss",
        "aud": ["api"],
        "exp": 9999999999,
        "iat": 0,
        "realm_access": {"roles": ["manager", "viewer"]},
    }
    claims = _parse_claims(payload)
    assert claims.has_role("manager") is True
    assert claims.has_role("viewer") is True


def test_parse_claims_extra_fields() -> None:
    """標準外フィールドが extra に格納されること。"""
    payload = {
        "sub": "u",
        "iss": "i",
        "aud": ["a"],
        "exp": 9,
        "iat": 0,
        "custom_field": "custom_value",
        "tenant_id": "tenant-123",
    }
    claims = _parse_claims(payload)
    assert claims.extra["custom_field"] == "custom_value"
    assert claims.extra["tenant_id"] == "tenant-123"


def test_auth_error_str() -> None:
    """AuthError.__str__ が code: message 形式であること。"""
    err = AuthError(code="TEST_CODE", message="test message")
    assert str(err) == "TEST_CODE: test message"


def test_auth_error_with_cause() -> None:
    """AuthError に cause が設定されること。"""
    cause = ValueError("original error")
    err = AuthError(code="TEST_CODE", message="wrapped", cause=cause)
    assert err.__cause__ is cause
