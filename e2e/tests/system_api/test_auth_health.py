"""Auth サービス ヘルスチェック E2E テスト。"""
import pytest
import requests


def _server_available(url):
    try:
        requests.get(url, timeout=2)
        return True
    except requests.ConnectionError:
        return False


class TestAuthHealth:
    """Auth サービスのヘルスエンドポイントを検証する。"""

    @pytest.mark.smoke
    def test_auth_healthz(self, auth_client):
        """GET /healthz が 200 と {"status": "ok"} を返す。"""
        url = auth_client.base_url + "/healthz"
        if not _server_available(url):
            pytest.skip("Auth server is not running")
        response = auth_client.get(url)
        assert response.status_code == 200
        data = response.json()
        assert data["status"] == "ok"

    @pytest.mark.smoke
    def test_auth_readyz(self, auth_client):
        """GET /readyz が 200 と {"status": "ready"} を返す。"""
        url = auth_client.base_url + "/readyz"
        if not _server_available(url):
            pytest.skip("Auth server is not running")
        response = auth_client.get(url)
        assert response.status_code == 200
        data = response.json()
        assert data["status"] == "ready"
