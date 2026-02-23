"""Auth サービス ユーザー管理 E2E テスト。"""

import pytest
import requests


def _server_available(url):
    try:
        requests.get(url, timeout=2)
        return True
    except requests.ConnectionError:
        return False


class TestAuthUsers:
    """Auth サービスのユーザー管理エンドポイントを検証する。"""

    @pytest.fixture(autouse=True)
    def _skip_if_unavailable(self, auth_client):
        if not _server_available(auth_client.base_url + "/healthz"):
            pytest.skip("Auth server is not running")

    def test_list_users(self, auth_client):
        """GET /api/v1/users がユーザー一覧を返す。"""
        response = auth_client.get(
            auth_client.base_url + "/api/v1/users",
        )
        assert response.status_code == 200
        data = response.json()
        assert "users" in data
        assert isinstance(data["users"], list)

    def test_list_users_with_pagination(self, auth_client):
        """GET /api/v1/users にページネーションパラメータが効く。"""
        response = auth_client.get(
            auth_client.base_url + "/api/v1/users",
            params={"page": 1, "per_page": 5},
        )
        assert response.status_code == 200
        data = response.json()
        assert "users" in data
        assert len(data["users"]) <= 5

    def test_list_users_with_search(self, auth_client):
        """GET /api/v1/users に検索クエリが効く。"""
        response = auth_client.get(
            auth_client.base_url + "/api/v1/users",
            params={"search": "admin"},
        )
        assert response.status_code == 200
        data = response.json()
        assert "users" in data

    def test_get_user_by_id(self, auth_client):
        """GET /api/v1/users/:id が特定ユーザーを返す。"""
        # まずユーザー一覧から ID を取得
        list_response = auth_client.get(
            auth_client.base_url + "/api/v1/users",
        )
        assert list_response.status_code == 200
        users = list_response.json()["users"]
        if not users:
            pytest.skip("No users available for testing")
        user_id = users[0]["id"]

        response = auth_client.get(
            auth_client.base_url + f"/api/v1/users/{user_id}",
        )
        assert response.status_code == 200
        data = response.json()
        assert data["id"] == user_id

    def test_get_user_not_found(self, auth_client):
        """GET /api/v1/users/:id に存在しない ID で 404 を返す。"""
        response = auth_client.get(
            auth_client.base_url + "/api/v1/users/nonexistent-user-id-000",
        )
        assert response.status_code == 404
