"""GraphQL Gateway サービス ヘルスチェック E2E テスト。"""

import pytest
import requests


def _server_available(url):
    try:
        requests.get(url, timeout=2)
        return True
    except requests.ConnectionError:
        return False


class TestGraphQLGatewayHealth:
    """GraphQL Gateway サービスのヘルスエンドポイントを検証する。"""

    @pytest.mark.smoke
    def test_graphql_gateway_healthz(self, graphql_gateway_client):
        """GET /healthz が 200 と {"status": "ok"} を返す。"""
        url = graphql_gateway_client.base_url + "/healthz"
        if not _server_available(url):
            pytest.skip("GraphQL Gateway server is not running")
        response = graphql_gateway_client.get(url)
        assert response.status_code == 200
        data = response.json()
        assert data["status"] == "ok"

    @pytest.mark.smoke
    def test_graphql_gateway_readyz(self, graphql_gateway_client):
        """GET /readyz が 200 を返す。"""
        url = graphql_gateway_client.base_url + "/readyz"
        if not _server_available(url):
            pytest.skip("GraphQL Gateway server is not running")
        response = graphql_gateway_client.get(url)
        assert response.status_code == 200
        data = response.json()
        assert data["status"] in ("ready", "ok")

    @pytest.mark.smoke
    def test_graphql_gateway_health_query(self, graphql_gateway_client):
        """POST /graphql に health クエリを送り正常なレスポンスが返る。"""
        url = graphql_gateway_client.base_url + "/graphql"
        if not _server_available(graphql_gateway_client.base_url + "/healthz"):
            pytest.skip("GraphQL Gateway server is not running")
        payload = {"query": "{ health { name status } }"}
        response = graphql_gateway_client.post(url, json=payload)
        assert response.status_code == 200
        data = response.json()
        assert "errors" not in data or len(data["errors"]) == 0
        assert data["data"]["health"]["status"] == "ok"
