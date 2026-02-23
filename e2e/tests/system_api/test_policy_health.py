"""Policy サービス ヘルスチェック E2E テスト。"""

import pytest
import requests


def _server_available(url):
    try:
        requests.get(url, timeout=2)
        return True
    except requests.ConnectionError:
        return False


class TestPolicyHealth:
    """Policy サービスのヘルスエンドポイントを検証する。"""

    @pytest.mark.smoke
    def test_policy_healthz(self, policy_client):
        """GET /healthz が 200 と {"status": "ok"} を返す。"""
        url = policy_client.base_url + "/healthz"
        if not _server_available(url):
            pytest.skip("Policy server is not running")
        response = policy_client.get(url)
        assert response.status_code == 200
        data = response.json()
        assert data["status"] == "ok"

    @pytest.mark.smoke
    def test_policy_readyz(self, policy_client):
        """GET /readyz が 200 を返す。"""
        url = policy_client.base_url + "/readyz"
        if not _server_available(url):
            pytest.skip("Policy server is not running")
        response = policy_client.get(url)
        assert response.status_code == 200
        data = response.json()
        assert data["status"] in ("ready", "ok")
