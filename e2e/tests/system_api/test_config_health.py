"""Config サービス ヘルスチェック E2E テスト。"""

import pytest
import requests


def _server_available(url):
    try:
        requests.get(url, timeout=2)
        return True
    except requests.ConnectionError:
        return False


class TestConfigHealth:
    """Config サービスのヘルスエンドポイントを検証する。"""

    @pytest.mark.smoke
    def test_config_healthz(self, config_client):
        """GET /healthz が 200 と {"status": "ok"} を返す。"""
        url = config_client.base_url + "/healthz"
        if not _server_available(url):
            pytest.skip("Config server is not running")
        response = config_client.get(url)
        assert response.status_code == 200
        data = response.json()
        assert data["status"] == "ok"

    @pytest.mark.smoke
    def test_config_readyz(self, config_client):
        """GET /readyz が 200 と {"status": "ready"} を返す。"""
        url = config_client.base_url + "/readyz"
        if not _server_available(url):
            pytest.skip("Config server is not running")
        response = config_client.get(url)
        assert response.status_code == 200
        data = response.json()
        assert data["status"] == "ready"
