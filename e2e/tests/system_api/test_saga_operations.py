"""Saga サービス CRUD E2E テスト。"""

import uuid

import pytest
import requests


def _server_available(url):
    try:
        requests.get(url, timeout=2)
        return True
    except requests.ConnectionError:
        return False


# テスト用ワークフロー YAML（最小構成）
_TEST_WORKFLOW_YAML = """\
name: test-workflow
steps:
  - name: step-1
    action: noop
    service: ""
    method: ""
    compensate_action: noop
    compensate_service: ""
    compensate_method: ""
"""


class TestWorkflowOperations:
    """Saga サービスのワークフロー管理エンドポイントを検証する。"""

    @pytest.fixture(autouse=True)
    def _skip_if_unavailable(self, saga_client):
        if not _server_available(saga_client.base_url + "/healthz"):
            pytest.skip("Saga server is not running")

    def test_register_workflow(self, saga_client):
        """POST /api/v1/workflows でワークフローを登録できる。"""
        response = saga_client.post(
            saga_client.base_url + "/api/v1/workflows",
            json={"workflow_yaml": _TEST_WORKFLOW_YAML},
        )
        assert response.status_code in (200, 201, 503)
        if response.status_code == 503:
            pytest.skip("Saga server database is not available")
        data = response.json()
        assert "name" in data

    def test_register_workflow_missing_yaml(self, saga_client):
        """POST /api/v1/workflows で workflow_yaml なしは 400/422 を返す。"""
        response = saga_client.post(
            saga_client.base_url + "/api/v1/workflows",
            json={},
        )
        assert response.status_code in (400, 422)

    def test_register_workflow_empty_yaml(self, saga_client):
        """POST /api/v1/workflows で workflow_yaml が空文字列は 400/422 を返す。"""
        response = saga_client.post(
            saga_client.base_url + "/api/v1/workflows",
            json={"workflow_yaml": ""},
        )
        assert response.status_code in (400, 422)

    def test_list_workflows(self, saga_client):
        """GET /api/v1/workflows がワークフロー一覧を返す。"""
        response = saga_client.get(
            saga_client.base_url + "/api/v1/workflows",
        )
        assert response.status_code in (200, 503)
        if response.status_code == 503:
            pytest.skip("Saga server database is not available")
        data = response.json()
        assert "workflows" in data
        assert isinstance(data["workflows"], list)


class TestSagaOperations:
    """Saga サービスの Saga CRUD エンドポイントを検証する。"""

    @pytest.fixture(autouse=True)
    def _skip_if_unavailable(self, saga_client):
        if not _server_available(saga_client.base_url + "/healthz"):
            pytest.skip("Saga server is not running")

    @pytest.fixture(scope="class")
    def registered_workflow(self, saga_client):
        """テスト用ワークフローを事前に登録する。未起動の場合は skip。"""
        if not _server_available(saga_client.base_url + "/healthz"):
            pytest.skip("Saga server is not running")
        resp = saga_client.post(
            saga_client.base_url + "/api/v1/workflows",
            json={"workflow_yaml": _TEST_WORKFLOW_YAML},
        )
        if resp.status_code == 503:
            pytest.skip("Saga server database is not available")
        assert resp.status_code in (200, 201)
        return resp.json()["name"]

    def test_start_saga(self, saga_client, registered_workflow):
        """POST /api/v1/sagas で Saga を開始できる。"""
        response = saga_client.post(
            saga_client.base_url + "/api/v1/sagas",
            json={"workflow_name": registered_workflow},
        )
        assert response.status_code in (200, 201, 503)
        if response.status_code == 503:
            pytest.skip("Saga server database is not available")
        data = response.json()
        assert "saga_id" in data
        assert "status" in data

    def test_start_saga_missing_workflow_name(self, saga_client):
        """POST /api/v1/sagas で workflow_name なしは 400/422 を返す。"""
        response = saga_client.post(
            saga_client.base_url + "/api/v1/sagas",
            json={},
        )
        assert response.status_code in (400, 422)

    def test_start_saga_empty_workflow_name(self, saga_client):
        """POST /api/v1/sagas で workflow_name が空文字列は 400/422 を返す。"""
        response = saga_client.post(
            saga_client.base_url + "/api/v1/sagas",
            json={"workflow_name": ""},
        )
        assert response.status_code in (400, 422)

    def test_list_sagas(self, saga_client):
        """GET /api/v1/sagas が Saga 一覧を返す。"""
        response = saga_client.get(
            saga_client.base_url + "/api/v1/sagas",
        )
        assert response.status_code in (200, 503)
        if response.status_code == 503:
            pytest.skip("Saga server database is not available")
        data = response.json()
        assert "sagas" in data
        assert isinstance(data["sagas"], list)
        assert "pagination" in data

    def test_list_sagas_with_filters(self, saga_client):
        """GET /api/v1/sagas にフィルターパラメータが効く。"""
        response = saga_client.get(
            saga_client.base_url + "/api/v1/sagas",
            params={"workflow_name": "test-workflow", "page": 1, "page_size": 5},
        )
        assert response.status_code in (200, 503)
        if response.status_code == 503:
            pytest.skip("Saga server database is not available")
        data = response.json()
        assert "sagas" in data
        assert "pagination" in data
        assert data["pagination"]["page_size"] == 5

    def test_get_saga_by_id(self, saga_client, registered_workflow):
        """GET /api/v1/sagas/:id が特定の Saga を返す。"""
        # まず Saga を開始して ID を取得
        start_resp = saga_client.post(
            saga_client.base_url + "/api/v1/sagas",
            json={"workflow_name": registered_workflow},
        )
        if start_resp.status_code == 503:
            pytest.skip("Saga server database is not available")
        assert start_resp.status_code in (200, 201)
        saga_id = start_resp.json()["saga_id"]

        response = saga_client.get(
            saga_client.base_url + f"/api/v1/sagas/{saga_id}",
        )
        assert response.status_code in (200, 503)
        if response.status_code == 503:
            pytest.skip("Saga server database is not available")
        data = response.json()
        assert "saga" in data
        assert data["saga"]["saga_id"] == saga_id
        assert "step_logs" in data

    def test_get_saga_invalid_uuid(self, saga_client):
        """GET /api/v1/sagas/:id に無効な UUID で 400/422 を返す。"""
        response = saga_client.get(
            saga_client.base_url + "/api/v1/sagas/invalid-uuid",
        )
        assert response.status_code in (400, 422)

    def test_get_saga_not_found(self, saga_client):
        """GET /api/v1/sagas/:id に存在しない UUID で 404 を返す。"""
        nonexistent_id = str(uuid.uuid4())
        response = saga_client.get(
            saga_client.base_url + f"/api/v1/sagas/{nonexistent_id}",
        )
        assert response.status_code in (404, 503)
        if response.status_code == 503:
            pytest.skip("Saga server database is not available")
