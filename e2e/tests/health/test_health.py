"""Health check E2E tests."""
import pytest


class TestHealth:
    """Health endpoint tests."""

    @pytest.mark.smoke
    def test_healthz_returns_200(self, api_client):
        """GET /healthz should return 200 OK."""
        response = api_client.get(f"{api_client.base_url}/healthz")
        assert response.status_code == 200

    @pytest.mark.smoke
    def test_healthz_response_body(self, api_client):
        """GET /healthz should return a valid health response."""
        response = api_client.get(f"{api_client.base_url}/healthz")
        data = response.json()
        assert "status" in data
        assert data["status"] == "ok"
