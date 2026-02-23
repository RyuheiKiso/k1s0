"""Kong API Gateway ルーティング E2E テスト。

Kong Proxy 経由で各バックエンドサービスへのルーティング、
レート制限、CORS ヘッダーを検証する。
"""

import pytest
import requests


def _kong_available(url):
    """Kong proxy が起動しているか確認する。"""
    try:
        requests.get(url, timeout=2)
        return True
    except requests.ConnectionError:
        return False


class TestKongHealthRouting:
    """Kong 経由のヘルスエンドポイントへのルーティングを検証する。"""

    @pytest.mark.smoke
    def test_healthz_via_kong(self, kong_client):
        """Kong proxy 経由で /healthz にアクセスできる。"""
        url = kong_client.base_url + "/healthz"
        if not _kong_available(url):
            pytest.skip("Kong proxy is not running")
        response = kong_client.get(url)
        assert response.status_code == 200
        data = response.json()
        assert data["status"] == "ok"

    @pytest.mark.smoke
    def test_readyz_via_kong(self, kong_client):
        """Kong proxy 経由で /readyz にアクセスできる。"""
        url = kong_client.base_url + "/readyz"
        if not _kong_available(url):
            pytest.skip("Kong proxy is not running")
        response = kong_client.get(url)
        assert response.status_code == 200
        data = response.json()
        assert data["status"] == "ready"


class TestKongAuthRouting:
    """Kong 経由の Auth API ルーティングを検証する。"""

    @pytest.mark.smoke
    def test_auth_api_routing(self, kong_client):
        """Kong proxy 経由で /api/v1/auth にルーティングされる。"""
        url = kong_client.base_url + "/api/v1/auth/token"
        if not _kong_available(kong_client.base_url + "/healthz"):
            pytest.skip("Kong proxy is not running")
        response = kong_client.get(url)
        # バックエンドが応答すること（404 でもルーティング自体は成功）
        assert response.status_code != 502
        assert response.status_code != 503

    @pytest.mark.smoke
    def test_users_api_routing(self, kong_client):
        """Kong proxy 経由で /api/v1/users にルーティングされる。"""
        url = kong_client.base_url + "/api/v1/users"
        if not _kong_available(kong_client.base_url + "/healthz"):
            pytest.skip("Kong proxy is not running")
        response = kong_client.get(url)
        assert response.status_code != 502
        assert response.status_code != 503

    @pytest.mark.smoke
    def test_audit_api_routing(self, kong_client):
        """Kong proxy 経由で /api/v1/audit にルーティングされる。"""
        url = kong_client.base_url + "/api/v1/audit"
        if not _kong_available(kong_client.base_url + "/healthz"):
            pytest.skip("Kong proxy is not running")
        response = kong_client.get(url)
        assert response.status_code != 502
        assert response.status_code != 503


class TestKongConfigRouting:
    """Kong 経由の Config API ルーティングを検証する。"""

    @pytest.mark.smoke
    def test_config_api_routing(self, kong_client):
        """Kong proxy 経由で /api/v1/config にルーティングされる。"""
        url = kong_client.base_url + "/api/v1/config"
        if not _kong_available(kong_client.base_url + "/healthz"):
            pytest.skip("Kong proxy is not running")
        response = kong_client.get(url)
        assert response.status_code != 502
        assert response.status_code != 503


class TestKongRateLimiting:
    """Kong のレート制限プラグインの動作を検証する。"""

    def test_rate_limit_headers_present(self, kong_client):
        """レスポンスに X-RateLimit-* ヘッダーが含まれる。"""
        url = kong_client.base_url + "/healthz"
        if not _kong_available(url):
            pytest.skip("Kong proxy is not running")
        response = kong_client.get(url)
        assert response.status_code == 200
        # Kong rate-limiting プラグインはレート制限ヘッダーを付与する
        rate_limit_headers = [
            h
            for h in response.headers
            if h.lower().startswith("x-ratelimit") or h.lower().startswith("ratelimit")
        ]
        assert len(rate_limit_headers) > 0, (
            f"Expected rate limit headers, got: {dict(response.headers)}"
        )


class TestKongCORS:
    """Kong の CORS プラグインの動作を検証する。"""

    def test_cors_preflight(self, kong_client):
        """OPTIONS リクエストに CORS ヘッダーが返る。"""
        url = kong_client.base_url + "/api/v1/auth"
        if not _kong_available(kong_client.base_url + "/healthz"):
            pytest.skip("Kong proxy is not running")
        response = requests.options(
            url,
            headers={
                "Origin": "http://localhost:3000",
                "Access-Control-Request-Method": "GET",
                "Access-Control-Request-Headers": "Authorization, Content-Type",
            },
        )
        assert "access-control-allow-origin" in {h.lower() for h in response.headers}, (
            f"Expected CORS headers, got: {dict(response.headers)}"
        )

    def test_cors_origin_header(self, kong_client):
        """GET リクエストに Access-Control-Allow-Origin が返る。"""
        url = kong_client.base_url + "/healthz"
        if not _kong_available(url):
            pytest.skip("Kong proxy is not running")
        response = requests.get(
            url,
            headers={"Origin": "http://localhost:3000"},
        )
        assert response.status_code == 200
        assert "access-control-allow-origin" in {h.lower() for h in response.headers}


class TestKongAdminAPI:
    """Kong Admin API の動作を検証する。"""

    def test_admin_api_accessible(self, kong_admin_url):
        """Kong Admin API が応答する。"""
        try:
            response = requests.get(kong_admin_url, timeout=2)
        except requests.ConnectionError:
            pytest.skip("Kong admin API is not running")
        assert response.status_code == 200

    def test_admin_api_services(self, kong_admin_url):
        """Admin API で登録済みサービスが確認できる。"""
        try:
            response = requests.get(f"{kong_admin_url}/services", timeout=2)
        except requests.ConnectionError:
            pytest.skip("Kong admin API is not running")
        assert response.status_code == 200
        data = response.json()
        service_names = [s["name"] for s in data.get("data", [])]
        assert "auth-v1" in service_names
        assert "config-v1" in service_names

    def test_admin_api_routes(self, kong_admin_url):
        """Admin API で登録済みルートが確認できる。"""
        try:
            response = requests.get(f"{kong_admin_url}/routes", timeout=2)
        except requests.ConnectionError:
            pytest.skip("Kong admin API is not running")
        assert response.status_code == 200
        data = response.json()
        route_names = [r["name"] for r in data.get("data", [])]
        assert "auth-token-route" in route_names
        assert "config-route" in route_names

    def test_admin_api_plugins(self, kong_admin_url):
        """Admin API でグローバルプラグインが確認できる。"""
        try:
            response = requests.get(f"{kong_admin_url}/plugins", timeout=2)
        except requests.ConnectionError:
            pytest.skip("Kong admin API is not running")
        assert response.status_code == 200
        data = response.json()
        plugin_names = [p["name"] for p in data.get("data", [])]
        assert "cors" in plugin_names
        assert "rate-limiting" in plugin_names
        assert "prometheus" in plugin_names


class TestKongNoMatchRoute:
    """Kong で未定義パスへのアクセスを検証する。"""

    def test_unknown_path_returns_404(self, kong_client):
        """未定義パスへのアクセスで 404 が返る。"""
        url = kong_client.base_url + "/api/v1/nonexistent"
        if not _kong_available(kong_client.base_url + "/healthz"):
            pytest.skip("Kong proxy is not running")
        response = kong_client.get(url)
        # Kong はマッチしないルートに対して 404 を返す
        assert response.status_code == 404
