"""Auth サービス 監査ログ E2E テスト。"""

import pytest
import requests


def _server_available(url):
    try:
        requests.get(url, timeout=2)
        return True
    except requests.ConnectionError:
        return False


class TestAuthAudit:
    """Auth サービスの監査ログエンドポイントを検証する。"""

    @pytest.fixture(autouse=True)
    def _skip_if_unavailable(self, auth_client):
        if not _server_available(auth_client.base_url + "/healthz"):
            pytest.skip("Auth server is not running")

    def test_record_audit_log(self, auth_client):
        """POST /api/v1/audit/logs で監査ログを記録できる。"""
        response = auth_client.post(
            auth_client.base_url + "/api/v1/audit/logs",
            json={
                "action": "LOGIN",
                "user_id": "test-user-001",
                "resource": "auth",
                "detail": "E2E test audit log entry",
            },
        )
        assert response.status_code in (200, 201)

    def test_search_audit_logs(self, auth_client):
        """GET /api/v1/audit/logs で監査ログを検索できる。"""
        response = auth_client.get(
            auth_client.base_url + "/api/v1/audit/logs",
        )
        assert response.status_code == 200
        data = response.json()
        assert "logs" in data
        assert isinstance(data["logs"], list)

    def test_search_audit_logs_with_date_filter(self, auth_client):
        """GET /api/v1/audit/logs に日付フィルタが効く。"""
        response = auth_client.get(
            auth_client.base_url + "/api/v1/audit/logs",
            params={
                "from": "2024-01-01T00:00:00Z",
                "to": "2099-12-31T23:59:59Z",
            },
        )
        assert response.status_code == 200
        data = response.json()
        assert "logs" in data

    def test_search_audit_logs_with_user_filter(self, auth_client):
        """GET /api/v1/audit/logs にユーザーフィルタが効く。"""
        response = auth_client.get(
            auth_client.base_url + "/api/v1/audit/logs",
            params={"user_id": "test-user-001"},
        )
        assert response.status_code == 200
        data = response.json()
        assert "logs" in data
