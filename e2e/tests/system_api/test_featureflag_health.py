"""FeatureFlag サービス ヘルスチェック E2E テスト。"""

import pytest
import requests


def _server_available(url):
    try:
        requests.get(url, timeout=2)
        return True
    except requests.ConnectionError:
        return False


class TestFeatureFlagHealth:
    """FeatureFlag サービスのヘルスエンドポイントを検証する。"""

    @pytest.mark.smoke
    def test_featureflag_healthz(self, featureflag_client):
        """GET /healthz が 200 と {"status": "ok"} を返す。"""
        url = featureflag_client.base_url + "/healthz"
        if not _server_available(url):
            pytest.skip("FeatureFlag server is not running")
        response = featureflag_client.get(url)
        assert response.status_code == 200
        data = response.json()
        assert data["status"] == "ok"

    @pytest.mark.smoke
    def test_featureflag_readyz(self, featureflag_client):
        """GET /readyz が 200 を返す。"""
        url = featureflag_client.base_url + "/readyz"
        if not _server_available(url):
            pytest.skip("FeatureFlag server is not running")
        response = featureflag_client.get(url)
        assert response.status_code == 200
        data = response.json()
        assert data["status"] in ("ready", "ok")

    @pytest.mark.smoke
    def test_featureflag_metrics(self, featureflag_client):
        """GET /metrics が 200 を返す。"""
        url = featureflag_client.base_url + "/metrics"
        if not _server_available(url):
            pytest.skip("FeatureFlag server is not running")
        response = featureflag_client.get(url)
        assert response.status_code == 200
