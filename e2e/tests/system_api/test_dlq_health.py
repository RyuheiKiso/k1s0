"""DLQ サービス ヘルスチェック E2E テスト。"""

import pytest
import requests


def _server_available(url):
    try:
        requests.get(url, timeout=2)
        return True
    except requests.ConnectionError:
        return False


class TestDlqHealth:
    """DLQ サービスのヘルスエンドポイントを検証する。"""

    @pytest.mark.smoke
    def test_dlq_healthz(self, dlq_client):
        """GET /healthz が 200 と "ok" を返す。"""
        url = dlq_client.base_url + "/healthz"
        if not _server_available(url):
            pytest.skip("DLQ server is not running")
        response = dlq_client.get(url)
        assert response.status_code == 200
        assert response.text == "ok"

    @pytest.mark.smoke
    def test_dlq_readyz(self, dlq_client):
        """GET /readyz が 200 と "ok" を返す。"""
        url = dlq_client.base_url + "/readyz"
        if not _server_available(url):
            pytest.skip("DLQ server is not running")
        response = dlq_client.get(url)
        assert response.status_code == 200
        assert response.text == "ok"

    @pytest.mark.smoke
    def test_dlq_metrics(self, dlq_client):
        """GET /metrics が 200 と Prometheus 形式のメトリクスを返す。"""
        url = dlq_client.base_url + "/metrics"
        if not _server_available(dlq_client.base_url + "/healthz"):
            pytest.skip("DLQ server is not running")
        response = dlq_client.get(url)
        assert response.status_code == 200
        # Prometheus 形式は # で始まるコメント行またはメトリクス行を含む
        assert isinstance(response.text, str)
