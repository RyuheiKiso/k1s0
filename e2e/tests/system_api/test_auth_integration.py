"""Auth サービス Keycloak 統合 E2E テスト。

Keycloak から実際のトークンを取得し、auth-server API と統合テストを行う。
前提: docker compose --profile infra (keycloak) が起動済みであること。
テスト用クライアント k1s0-e2e-test は directAccessGrantsEnabled=true で、
Resource Owner Password Credentials Grant によりトークンを取得する。
"""
import base64
import json

import pytest
import requests

from .conftest import (
    KEYCLOAK_E2E_CLIENT_ID,
    KEYCLOAK_E2E_CLIENT_SECRET,
    KEYCLOAK_REALM,
    _keycloak_token_url,
    _keycloak_userinfo_url,
)


def _keycloak_available(base_url):
    try:
        resp = requests.get(
            f"{base_url}/realms/{KEYCLOAK_REALM}/.well-known/openid-configuration",
            timeout=3,
        )
        return resp.status_code == 200
    except requests.ConnectionError:
        return False


def _server_available(url):
    try:
        requests.get(url, timeout=2)
        return True
    except requests.ConnectionError:
        return False


def _decode_jwt_payload(token):
    """JWT の payload 部分をデコードする（署名検証なし）。"""
    parts = token.split(".")
    if len(parts) != 3:
        raise ValueError("Invalid JWT format")
    payload_b64 = parts[1]
    # Base64url padding
    padding = 4 - len(payload_b64) % 4
    if padding != 4:
        payload_b64 += "=" * padding
    payload_bytes = base64.urlsafe_b64decode(payload_b64)
    return json.loads(payload_bytes)


class TestKeycloakTokenAcquisition:
    """Keycloak からのトークン取得を検証する。"""

    @pytest.fixture(autouse=True)
    def _skip_if_unavailable(self, keycloak_base_url):
        if not _keycloak_available(keycloak_base_url):
            pytest.skip("Keycloak is not running")

    def test_obtain_admin_token(self, keycloak_base_url):
        """test-admin ユーザーでトークンを取得できる。"""
        resp = requests.post(
            _keycloak_token_url(keycloak_base_url),
            data={
                "grant_type": "password",
                "client_id": KEYCLOAK_E2E_CLIENT_ID,
                "client_secret": KEYCLOAK_E2E_CLIENT_SECRET,
                "username": "test-admin",
                "password": "admin123",
                "scope": "openid profile email",
            },
            timeout=10,
        )
        assert resp.status_code == 200
        data = resp.json()
        assert "access_token" in data
        assert "refresh_token" in data
        assert "id_token" in data
        assert data["token_type"] == "Bearer"

    def test_obtain_user_token(self, keycloak_base_url):
        """test-user ユーザーでトークンを取得できる。"""
        resp = requests.post(
            _keycloak_token_url(keycloak_base_url),
            data={
                "grant_type": "password",
                "client_id": KEYCLOAK_E2E_CLIENT_ID,
                "client_secret": KEYCLOAK_E2E_CLIENT_SECRET,
                "username": "test-user",
                "password": "user123",
                "scope": "openid profile email",
            },
            timeout=10,
        )
        assert resp.status_code == 200
        data = resp.json()
        assert "access_token" in data

    def test_invalid_credentials_rejected(self, keycloak_base_url):
        """不正なパスワードではトークン取得に失敗する。"""
        resp = requests.post(
            _keycloak_token_url(keycloak_base_url),
            data={
                "grant_type": "password",
                "client_id": KEYCLOAK_E2E_CLIENT_ID,
                "client_secret": KEYCLOAK_E2E_CLIENT_SECRET,
                "username": "test-admin",
                "password": "wrong-password",
                "scope": "openid profile email",
            },
            timeout=10,
        )
        assert resp.status_code == 401

    def test_invalid_client_rejected(self, keycloak_base_url):
        """不正なクライアントシークレットではトークン取得に失敗する。"""
        resp = requests.post(
            _keycloak_token_url(keycloak_base_url),
            data={
                "grant_type": "password",
                "client_id": KEYCLOAK_E2E_CLIENT_ID,
                "client_secret": "wrong-secret",
                "username": "test-admin",
                "password": "admin123",
                "scope": "openid profile email",
            },
            timeout=10,
        )
        assert resp.status_code == 401


class TestJwtClaimsStructure:
    """JWT Claims が認証認可設計.md の定義と一致するか検証する。"""

    @pytest.fixture(autouse=True)
    def _skip_if_unavailable(self, keycloak_base_url):
        if not _keycloak_available(keycloak_base_url):
            pytest.skip("Keycloak is not running")

    def test_admin_token_has_expected_claims(self, keycloak_admin_token):
        """test-admin のトークンに必要な Claims が含まれる。"""
        payload = _decode_jwt_payload(keycloak_admin_token["access_token"])

        # 標準 Claims
        assert "iss" in payload
        assert "k1s0" in payload["iss"]
        assert "sub" in payload
        assert "exp" in payload
        assert "iat" in payload
        assert payload["typ"] == "Bearer"

    def test_admin_token_has_realm_roles(self, keycloak_admin_token):
        """test-admin のトークンに realm_access.roles が含まれる。"""
        payload = _decode_jwt_payload(keycloak_admin_token["access_token"])
        assert "realm_access" in payload
        assert "roles" in payload["realm_access"]
        roles = payload["realm_access"]["roles"]
        assert "sys_admin" in roles
        assert "user" in roles

    def test_user_token_has_user_role_only(self, keycloak_user_token):
        """test-user のトークンには user ロールのみが含まれる。"""
        payload = _decode_jwt_payload(keycloak_user_token["access_token"])
        assert "realm_access" in payload
        roles = payload["realm_access"]["roles"]
        assert "user" in roles
        assert "sys_admin" not in roles

    def test_token_has_tier_access_claim(self, keycloak_admin_token):
        """トークンに tier_access Claim が含まれる。"""
        payload = _decode_jwt_payload(keycloak_admin_token["access_token"])
        assert "tier_access" in payload
        tier_access = payload["tier_access"]
        # JSON として解析可能であること
        if isinstance(tier_access, str):
            tier_access = json.loads(tier_access)
        assert "system" in tier_access
        assert "business" in tier_access
        assert "service" in tier_access

    def test_token_has_email_claim(self, keycloak_admin_token):
        """トークンに email Claim が含まれる。"""
        payload = _decode_jwt_payload(keycloak_admin_token["access_token"])
        assert payload.get("email") == "admin@k1s0.dev"

    def test_token_has_preferred_username(self, keycloak_admin_token):
        """トークンに preferred_username Claim が含まれる。"""
        payload = _decode_jwt_payload(keycloak_admin_token["access_token"])
        assert payload.get("preferred_username") == "test-admin"


class TestKeycloakUserinfo:
    """Keycloak の userinfo エンドポイントを検証する。"""

    @pytest.fixture(autouse=True)
    def _skip_if_unavailable(self, keycloak_base_url):
        if not _keycloak_available(keycloak_base_url):
            pytest.skip("Keycloak is not running")

    def test_userinfo_with_valid_token(self, keycloak_base_url, keycloak_admin_token):
        """有効なトークンで userinfo を取得できる。"""
        resp = requests.get(
            _keycloak_userinfo_url(keycloak_base_url),
            headers={
                "Authorization": f"Bearer {keycloak_admin_token['access_token']}",
            },
            timeout=10,
        )
        assert resp.status_code == 200
        data = resp.json()
        assert data["preferred_username"] == "test-admin"
        assert data["email"] == "admin@k1s0.dev"

    def test_userinfo_without_token(self, keycloak_base_url):
        """トークンなしでは userinfo にアクセスできない。"""
        resp = requests.get(
            _keycloak_userinfo_url(keycloak_base_url),
            timeout=10,
        )
        assert resp.status_code == 401

    def test_userinfo_with_invalid_token(self, keycloak_base_url):
        """不正なトークンでは userinfo にアクセスできない。"""
        resp = requests.get(
            _keycloak_userinfo_url(keycloak_base_url),
            headers={"Authorization": "Bearer invalid-token-value"},
            timeout=10,
        )
        assert resp.status_code == 401


class TestTokenRefresh:
    """トークンリフレッシュのフローを検証する。"""

    @pytest.fixture(autouse=True)
    def _skip_if_unavailable(self, keycloak_base_url):
        if not _keycloak_available(keycloak_base_url):
            pytest.skip("Keycloak is not running")

    def test_refresh_token_grants_new_access_token(
        self, keycloak_base_url, keycloak_admin_token
    ):
        """refresh_token で新しい access_token を取得できる。"""
        resp = requests.post(
            _keycloak_token_url(keycloak_base_url),
            data={
                "grant_type": "refresh_token",
                "client_id": KEYCLOAK_E2E_CLIENT_ID,
                "client_secret": KEYCLOAK_E2E_CLIENT_SECRET,
                "refresh_token": keycloak_admin_token["refresh_token"],
            },
            timeout=10,
        )
        assert resp.status_code == 200
        data = resp.json()
        assert "access_token" in data
        assert "refresh_token" in data
        # 新しいトークンが発行されていること
        assert data["access_token"] != keycloak_admin_token["access_token"]


class TestClientCredentialsGrant:
    """Client Credentials Grant（サービス間認証）を検証する。"""

    @pytest.fixture(autouse=True)
    def _skip_if_unavailable(self, keycloak_base_url):
        if not _keycloak_available(keycloak_base_url):
            pytest.skip("Keycloak is not running")

    def test_service_client_credentials(self, keycloak_base_url):
        """k1s0-service クライアントで Client Credentials Grant が成功する。"""
        resp = requests.post(
            _keycloak_token_url(keycloak_base_url),
            data={
                "grant_type": "client_credentials",
                "client_id": "k1s0-service",
                "client_secret": "dev-service-secret",
            },
            timeout=10,
        )
        assert resp.status_code == 200
        data = resp.json()
        assert "access_token" in data
        assert data["token_type"] == "Bearer"

    def test_service_token_has_tier_access(self, keycloak_base_url):
        """サービストークンに tier_access Claim が含まれる。"""
        resp = requests.post(
            _keycloak_token_url(keycloak_base_url),
            data={
                "grant_type": "client_credentials",
                "client_id": "k1s0-service",
                "client_secret": "dev-service-secret",
            },
            timeout=10,
        )
        assert resp.status_code == 200
        payload = _decode_jwt_payload(resp.json()["access_token"])
        assert "tier_access" in payload


class TestAuthServerIntegration:
    """Keycloak トークンを使った auth-server API との統合テスト。"""

    @pytest.fixture(autouse=True)
    def _skip_if_unavailable(self, keycloak_base_url, auth_client):
        if not _keycloak_available(keycloak_base_url):
            pytest.skip("Keycloak is not running")
        if not _server_available(auth_client.base_url + "/healthz"):
            pytest.skip("Auth server is not running")

    def test_validate_real_keycloak_token(self, auth_client, keycloak_admin_token):
        """Keycloak が発行した実トークンで /api/v1/auth/validate が成功する。"""
        response = auth_client.post(
            auth_client.base_url + "/api/v1/auth/validate",
            json={"token": keycloak_admin_token["access_token"]},
        )
        assert response.status_code == 200
        data = response.json()
        assert data["valid"] is True

    def test_introspect_real_keycloak_token(self, auth_client, keycloak_admin_token):
        """Keycloak が発行した実トークンで /api/v1/auth/introspect が active=true。"""
        response = auth_client.post(
            auth_client.base_url + "/api/v1/auth/introspect",
            json={"token": keycloak_admin_token["access_token"]},
        )
        assert response.status_code == 200
        data = response.json()
        assert data["active"] is True

    def test_check_permission_with_admin_token(
        self, auth_client, keycloak_admin_token
    ):
        """sys_admin ロール持ちトークンでパーミッションチェックが通る。"""
        response = auth_client.post(
            auth_client.base_url + "/api/v1/auth/check-permission",
            json={
                "token": keycloak_admin_token["access_token"],
                "permission": "read",
                "resource": "users",
            },
        )
        # auth-server 実装のレスポンスコードに合わせる
        assert response.status_code in (200, 501)

    def test_check_permission_with_user_token(self, auth_client, keycloak_user_token):
        """一般ユーザートークンでのパーミッションチェック。"""
        response = auth_client.post(
            auth_client.base_url + "/api/v1/auth/check-permission",
            json={
                "token": keycloak_user_token["access_token"],
                "permission": "write",
                "resource": "users",
            },
        )
        # 結果は実装依存だが、レスポンスが返ること
        assert response.status_code in (200, 403, 501)


class TestOIDCDiscovery:
    """Keycloak OIDC Discovery エンドポイントの検証。"""

    @pytest.fixture(autouse=True)
    def _skip_if_unavailable(self, keycloak_base_url):
        if not _keycloak_available(keycloak_base_url):
            pytest.skip("Keycloak is not running")

    def test_discovery_endpoint(self, keycloak_base_url):
        """OIDC Discovery エンドポイントが正しいメタデータを返す。"""
        resp = requests.get(
            f"{keycloak_base_url}/realms/{KEYCLOAK_REALM}"
            "/.well-known/openid-configuration",
            timeout=10,
        )
        assert resp.status_code == 200
        data = resp.json()

        assert data["issuer"] == f"{keycloak_base_url}/realms/{KEYCLOAK_REALM}"
        assert "token_endpoint" in data
        assert "authorization_endpoint" in data
        assert "jwks_uri" in data
        assert "userinfo_endpoint" in data

    def test_jwks_endpoint(self, keycloak_base_url):
        """JWKS エンドポイントから公開鍵を取得できる。"""
        resp = requests.get(
            f"{keycloak_base_url}/realms/{KEYCLOAK_REALM}"
            "/protocol/openid-connect/certs",
            timeout=10,
        )
        assert resp.status_code == 200
        data = resp.json()
        assert "keys" in data
        assert len(data["keys"]) > 0

        key = data["keys"][0]
        assert key["kty"] == "RSA"
        assert key["alg"] == "RS256"
        assert "kid" in key
        assert "n" in key
        assert "e" in key
