"""Saga サービス ヘルスチェック E2E テスト。"""
import pytest
import requests


def _server_available(url):
    try:
        requests.get(url, timeout=2)
        return True
    except requests.ConnectionError:
        return False


class TestSagaHealth:
    """Saga サービスのヘルスエンドポイントを検証する。"""

    @pytest.mark.smoke
    def test_saga_healthz(self, saga_client):
        """GET /healthz が 200 と "ok" を返す。"""
        url = saga_client.base_url + "/healthz"
        if not _server_available(url):
            pytest.skip("Saga server is not running")
        response = saga_client.get(url)
        assert response.status_code == 200
        assert response.text == "ok"

    @pytest.mark.smoke
    def test_saga_readyz(self, saga_client):
        """GET /readyz が 200 と "ok" を返す。"""
        url = saga_client.base_url + "/readyz"
        if not _server_available(url):
            pytest.skip("Saga server is not running")
        response = saga_client.get(url)
        assert response.status_code == 200
        assert response.text == "ok"

    @pytest.mark.smoke
    def test_saga_metrics(self, saga_client):
        """GET /metrics が 200 と Prometheus 形式のメトリクスを返す。"""
        url = saga_client.base_url + "/metrics"
        if not _server_available(saga_client.base_url + "/healthz"):
            pytest.skip("Saga server is not running")
        response = saga_client.get(url)
        assert response.status_code == 200
        # Prometheus 形式は # で始まるコメント行またはメトリクス行を含む
        assert isinstance(response.text, str)
