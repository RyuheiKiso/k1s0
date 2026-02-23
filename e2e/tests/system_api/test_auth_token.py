"""Auth サービス トークン検証 E2E テスト。"""

import pytest
import requests


def _server_available(url):
    try:
        requests.get(url, timeout=2)
        return True
    except requests.ConnectionError:
        return False


class TestAuthTokenValidation:
    """Auth サービスのトークン検証エンドポイントを検証する。"""

    @pytest.fixture(autouse=True)
    def _skip_if_unavailable(self, auth_client):
        if not _server_available(auth_client.base_url + "/healthz"):
            pytest.skip("Auth server is not running")

    def test_validate_token_with_valid_jwt(self, auth_client):
        """有効な JWT トークンで /api/v1/auth/validate が 200 を返す。"""
        response = auth_client.post(
            auth_client.base_url + "/api/v1/auth/validate",
            json={"token": "valid-test-token"},
        )
        assert response.status_code == 200
        data = response.json()
        assert "valid" in data
        assert data["valid"] is True

    def test_validate_token_with_expired_jwt(self, auth_client):
        """期限切れ JWT で /api/v1/auth/validate が 401 を返す。"""
        response = auth_client.post(
            auth_client.base_url + "/api/v1/auth/validate",
            json={"token": "expired-test-token"},
        )
        assert response.status_code == 401
        data = response.json()
        assert "valid" in data
        assert data["valid"] is False

    def test_validate_token_with_invalid_jwt(self, auth_client):
        """不正な JWT で /api/v1/auth/validate が 401 を返す。"""
        response = auth_client.post(
            auth_client.base_url + "/api/v1/auth/validate",
            json={"token": "invalid-garbage-token"},
        )
        assert response.status_code == 401

    def test_introspect_token_active(self, auth_client):
        """アクティブトークンの /api/v1/auth/introspect が active=true を返す。"""
        response = auth_client.post(
            auth_client.base_url + "/api/v1/auth/introspect",
            json={"token": "valid-test-token"},
        )
        assert response.status_code == 200
        data = response.json()
        assert data["active"] is True

    def test_introspect_token_inactive(self, auth_client):
        """非アクティブトークンの /api/v1/auth/introspect が active=false を返す。"""
        response = auth_client.post(
            auth_client.base_url + "/api/v1/auth/introspect",
            json={"token": "expired-test-token"},
        )
        assert response.status_code == 200
        data = response.json()
        assert data["active"] is False
