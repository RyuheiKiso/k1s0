"""Config サービス CRUD E2E テスト。"""

import uuid

import pytest
import requests


def _server_available(url):
    try:
        requests.get(url, timeout=2)
        return True
    except requests.ConnectionError:
        return False


class TestConfigCRUD:
    """Config サービスの設定 CRUD エンドポイントを検証する。"""

    NAMESPACE = "e2e-test"

    @pytest.fixture(autouse=True)
    def _skip_if_unavailable(self, config_client):
        if not _server_available(config_client.base_url + "/healthz"):
            pytest.skip("Config server is not running")

    @pytest.fixture()
    def unique_key(self):
        return f"test-key-{uuid.uuid4().hex[:8]}"

    def test_create_config(self, config_client, unique_key):
        """PUT /api/v1/config/:namespace/:key で設定を作成できる。"""
        response = config_client.put(
            config_client.base_url + f"/api/v1/config/{self.NAMESPACE}/{unique_key}",
            json={"value": "test-value", "description": "E2E test config"},
        )
        assert response.status_code in (200, 201)

    def test_get_config(self, config_client, unique_key):
        """GET /api/v1/config/:namespace/:key で設定を取得できる。"""
        # 作成
        config_client.put(
            config_client.base_url + f"/api/v1/config/{self.NAMESPACE}/{unique_key}",
            json={"value": "get-test-value"},
        )
        # 取得
        response = config_client.get(
            config_client.base_url + f"/api/v1/config/{self.NAMESPACE}/{unique_key}",
        )
        assert response.status_code == 200
        data = response.json()
        assert data["value"] == "get-test-value"

    def test_get_config_not_found(self, config_client):
        """GET /api/v1/config/:namespace/:key に存在しないキーで 404 を返す。"""
        response = config_client.get(
            config_client.base_url + f"/api/v1/config/{self.NAMESPACE}/nonexistent-key-000",
        )
        assert response.status_code == 404

    def test_list_configs(self, config_client, unique_key):
        """GET /api/v1/config/:namespace で設定一覧を取得できる。"""
        # 作成
        config_client.put(
            config_client.base_url + f"/api/v1/config/{self.NAMESPACE}/{unique_key}",
            json={"value": "list-test-value"},
        )
        # 一覧取得
        response = config_client.get(
            config_client.base_url + f"/api/v1/config/{self.NAMESPACE}",
        )
        assert response.status_code == 200
        data = response.json()
        assert "configs" in data
        assert isinstance(data["configs"], list)

    def test_list_configs_with_search(self, config_client):
        """GET /api/v1/config/:namespace に検索クエリが効く。"""
        response = config_client.get(
            config_client.base_url + f"/api/v1/config/{self.NAMESPACE}",
            params={"search": "test"},
        )
        assert response.status_code == 200
        data = response.json()
        assert "configs" in data

    def test_update_config(self, config_client, unique_key):
        """PUT /api/v1/config/:namespace/:key で設定を更新できる。"""
        url = config_client.base_url + f"/api/v1/config/{self.NAMESPACE}/{unique_key}"
        # 作成
        create_resp = config_client.put(url, json={"value": "original"})
        assert create_resp.status_code in (200, 201)
        version = create_resp.json().get("version", 1)

        # 更新
        update_resp = config_client.put(
            url,
            json={"value": "updated", "version": version},
        )
        assert update_resp.status_code == 200

        # 確認
        get_resp = config_client.get(url)
        assert get_resp.json()["value"] == "updated"

    def test_update_config_version_conflict(self, config_client, unique_key):
        """PUT /api/v1/config/:namespace/:key でバージョン不一致時に 409 を返す。"""
        url = config_client.base_url + f"/api/v1/config/{self.NAMESPACE}/{unique_key}"
        # 作成
        config_client.put(url, json={"value": "v1"})

        # 古いバージョンで更新を試行
        response = config_client.put(
            url,
            json={"value": "conflict", "version": 999},
        )
        assert response.status_code == 409

    def test_delete_config(self, config_client, unique_key):
        """DELETE /api/v1/config/:namespace/:key で設定を削除できる。"""
        url = config_client.base_url + f"/api/v1/config/{self.NAMESPACE}/{unique_key}"
        # 作成
        config_client.put(url, json={"value": "to-delete"})

        # 削除
        response = config_client.delete(url)
        assert response.status_code in (200, 204)

        # 削除確認
        get_resp = config_client.get(url)
        assert get_resp.status_code == 404

    def test_delete_config_not_found(self, config_client):
        """DELETE /api/v1/config/:namespace/:key に存在しないキーで 404 を返す。"""
        response = config_client.delete(
            config_client.base_url + f"/api/v1/config/{self.NAMESPACE}/nonexistent-key-000",
        )
        assert response.status_code == 404

    def test_get_service_config(self, config_client):
        """GET /api/v1/service-config でサービス設定を取得できる。"""
        response = config_client.get(
            config_client.base_url + "/api/v1/service-config",
        )
        assert response.status_code == 200
        data = response.json()
        assert isinstance(data, dict)
