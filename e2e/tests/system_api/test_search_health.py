"""Search サービス ヘルスチェック E2E テスト。"""

import pytest
import requests


def _server_available(url):
    try:
        requests.get(url, timeout=2)
        return True
    except requests.ConnectionError:
        return False


class TestSearchHealth:
    """Search サービスのヘルスエンドポイントを検証する。"""

    @pytest.mark.smoke
    def test_search_healthz(self, search_client):
        """GET /healthz が 200 と {"status": "ok"} を返す。"""
        url = search_client.base_url + "/healthz"
        if not _server_available(url):
            pytest.skip("Search server is not running")
        response = search_client.get(url)
        assert response.status_code == 200
        data = response.json()
        assert data["status"] == "ok"

    @pytest.mark.smoke
    def test_search_readyz(self, search_client):
        """GET /readyz が 200 を返す。"""
        url = search_client.base_url + "/readyz"
        if not _server_available(url):
            pytest.skip("Search server is not running")
        response = search_client.get(url)
        assert response.status_code == 200
        data = response.json()
        assert data["status"] in ("ready", "ok")
