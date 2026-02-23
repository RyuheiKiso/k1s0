"""Health check E2E tests."""

import pytest
import requests


class TestHealth:
    """Health endpoint tests."""

    @pytest.mark.smoke
    def test_healthz_returns_200(self, api_client):
        """GET /healthz should return 200 OK."""
        try:
            response = api_client.get(f"{api_client.base_url}/healthz")
        except requests.exceptions.ConnectionError:
            pytest.skip("サービスが起動していません（接続エラー）")
        assert response.status_code == 200

    @pytest.mark.smoke
    def test_healthz_response_body(self, api_client):
        """GET /healthz should return a valid health response."""
        try:
            response = api_client.get(f"{api_client.base_url}/healthz")
        except requests.exceptions.ConnectionError:
            pytest.skip("サービスが起動していません（接続エラー）")
        data = response.json()
        assert "status" in data
        assert data["status"] == "ok"
