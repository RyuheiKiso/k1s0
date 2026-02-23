"""Vault サービス ヘルスチェック E2E テスト。"""

import pytest
import requests


def _server_available(url):
    try:
        requests.get(url, timeout=2)
        return True
    except requests.ConnectionError:
        return False


class TestVaultHealth:
    """Vault サービスのヘルスエンドポイントを検証する。"""

    @pytest.mark.smoke
    def test_vault_healthz(self, vault_client):
        """GET /healthz が 200 と {"status": "ok"} を返す。"""
        url = vault_client.base_url + "/healthz"
        if not _server_available(url):
            pytest.skip("Vault server is not running")
        response = vault_client.get(url)
        assert response.status_code == 200
        data = response.json()
        assert data["status"] == "ok"

    @pytest.mark.smoke
    def test_vault_readyz(self, vault_client):
        """GET /readyz が 200 を返す。"""
        url = vault_client.base_url + "/readyz"
        if not _server_available(url):
            pytest.skip("Vault server is not running")
        response = vault_client.get(url)
        assert response.status_code == 200
        data = response.json()
        assert data["status"] in ("ready", "ok")

    @pytest.mark.smoke
    def test_vault_metrics(self, vault_client):
        """GET /metrics が 200 を返す。"""
        url = vault_client.base_url + "/metrics"
        if not _server_available(url):
            pytest.skip("Vault server is not running")
        response = vault_client.get(url)
        assert response.status_code == 200
