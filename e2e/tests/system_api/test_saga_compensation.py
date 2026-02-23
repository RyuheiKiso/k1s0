"""Saga 補償フロー E2E テスト。

補償トランザクションに関連する API の振る舞いを検証する。
Saga の status フィルタ、detail レスポンス構造、キャンセルフローを確認する。
"""

import uuid

import pytest
import requests


def _server_available(url):
    try:
        requests.get(url, timeout=2)
        return True
    except requests.ConnectionError:
        return False


_TEST_WORKFLOW_YAML = """\
name: compensation-test-workflow
steps:
  - name: step-1
    action: noop
    service: ""
    method: ""
    compensate_action: noop
    compensate_service: ""
    compensate_method: ""
  - name: step-2
    action: noop
    service: ""
    method: ""
    compensate_action: noop
    compensate_service: ""
    compensate_method: ""
"""


class TestSagaCompensationFields:
    """Saga の補償フロー関連フィールドを検証する。"""

    @pytest.fixture(autouse=True)
    def _skip_if_unavailable(self, saga_client):
        if not _server_available(saga_client.base_url + "/healthz"):
            pytest.skip("Saga server is not running")

    @pytest.fixture(scope="class")
    def registered_workflow(self, saga_client):
        """テスト用ワークフローを登録する。"""
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

    def test_saga_detail_contains_compensation_fields(self, saga_client, registered_workflow):
        """GET /api/v1/sagas/:id のレスポンスが補償フィールドを含む。"""
        start_resp = saga_client.post(
            saga_client.base_url + "/api/v1/sagas",
            json={"workflow_name": registered_workflow},
        )
        if start_resp.status_code == 503:
            pytest.skip("Saga server database is not available")
        assert start_resp.status_code in (200, 201)
        saga_id = start_resp.json()["saga_id"]

        detail_resp = saga_client.get(
            saga_client.base_url + f"/api/v1/sagas/{saga_id}",
        )
        assert detail_resp.status_code in (200, 503)
        if detail_resp.status_code == 503:
            pytest.skip("Saga server database is not available")
        data = detail_resp.json()
        assert "saga" in data
        assert "step_logs" in data
        saga = data["saga"]
        # 補償フィールドが存在することを確認（初期状態は null）
        assert "error_message" in saga
        assert saga["error_message"] is None
        assert isinstance(data["step_logs"], list)

    def test_saga_initial_status_is_started_or_running(self, saga_client, registered_workflow):
        """新規 Saga の status は STARTED, RUNNING, COMPLETED のいずれか。"""
        start_resp = saga_client.post(
            saga_client.base_url + "/api/v1/sagas",
            json={"workflow_name": registered_workflow},
        )
        if start_resp.status_code == 503:
            pytest.skip("Saga server database is not available")
        assert start_resp.status_code in (200, 201)
        data = start_resp.json()
        assert "saga_id" in data
        assert data["status"] in ("STARTED", "RUNNING", "COMPLETED")

    def test_list_sagas_status_filter_compensating(self, saga_client):
        """GET /api/v1/sagas?status=COMPENSATING が 200 を返す。"""
        response = saga_client.get(
            saga_client.base_url + "/api/v1/sagas",
            params={"status": "COMPENSATING"},
        )
        assert response.status_code in (200, 503)
        if response.status_code == 503:
            pytest.skip("Saga server database is not available")
        data = response.json()
        assert "sagas" in data
        assert "pagination" in data
        assert isinstance(data["sagas"], list)

    def test_list_sagas_status_filter_failed(self, saga_client):
        """GET /api/v1/sagas?status=FAILED が 200 を返す。"""
        response = saga_client.get(
            saga_client.base_url + "/api/v1/sagas",
            params={"status": "FAILED"},
        )
        assert response.status_code in (200, 503)
        if response.status_code == 503:
            pytest.skip("Saga server database is not available")
        data = response.json()
        assert "sagas" in data

    def test_list_sagas_all_valid_status_values_accepted(self, saga_client):
        """全ての有効な status フィルタ値が受け付けられる。"""
        for status in ["STARTED", "RUNNING", "COMPLETED", "COMPENSATING", "FAILED", "CANCELLED"]:
            response = saga_client.get(
                saga_client.base_url + "/api/v1/sagas",
                params={"status": status},
            )
            assert response.status_code in (200, 503), (
                f"status={status} returned unexpected {response.status_code}"
            )
            if response.status_code == 503:
                pytest.skip("Saga server database is not available")

    def test_saga_correlation_id_roundtrip(self, saga_client, registered_workflow):
        """correlation_id を指定して開始した Saga がそれを返す。"""
        correlation_id = str(uuid.uuid4())
        start_resp = saga_client.post(
            saga_client.base_url + "/api/v1/sagas",
            json={
                "workflow_name": registered_workflow,
                "correlation_id": correlation_id,
            },
        )
        if start_resp.status_code == 503:
            pytest.skip("Saga server database is not available")
        assert start_resp.status_code in (200, 201)
        saga_id = start_resp.json()["saga_id"]

        detail_resp = saga_client.get(
            saga_client.base_url + f"/api/v1/sagas/{saga_id}",
        )
        assert detail_resp.status_code in (200, 503)
        if detail_resp.status_code == 503:
            pytest.skip("Saga server database is not available")
        data = detail_resp.json()
        assert data["saga"]["correlation_id"] == correlation_id

    def test_list_sagas_correlation_id_filter(self, saga_client, registered_workflow):
        """GET /api/v1/sagas?correlation_id=xxx が特定の Saga を返す。"""
        correlation_id = str(uuid.uuid4())
        start_resp = saga_client.post(
            saga_client.base_url + "/api/v1/sagas",
            json={
                "workflow_name": registered_workflow,
                "correlation_id": correlation_id,
            },
        )
        if start_resp.status_code == 503:
            pytest.skip("Saga server database is not available")
        assert start_resp.status_code in (200, 201)

        list_resp = saga_client.get(
            saga_client.base_url + "/api/v1/sagas",
            params={"correlation_id": correlation_id},
        )
        assert list_resp.status_code in (200, 503)
        if list_resp.status_code == 503:
            pytest.skip("Saga server database is not available")
        data = list_resp.json()
        assert len(data["sagas"]) >= 1
        assert data["sagas"][0]["correlation_id"] == correlation_id

    def test_saga_step_logs_structure(self, saga_client, registered_workflow):
        """GET /api/v1/sagas/:id の step_logs がスキーマ通りの構造を持つ。"""
        start_resp = saga_client.post(
            saga_client.base_url + "/api/v1/sagas",
            json={"workflow_name": registered_workflow},
        )
        if start_resp.status_code == 503:
            pytest.skip("Saga server database is not available")
        assert start_resp.status_code in (200, 201)
        saga_id = start_resp.json()["saga_id"]

        detail_resp = saga_client.get(
            saga_client.base_url + f"/api/v1/sagas/{saga_id}",
        )
        assert detail_resp.status_code in (200, 503)
        if detail_resp.status_code == 503:
            pytest.skip("Saga server database is not available")
        data = detail_resp.json()
        # ステップログが存在する場合はフィールドを検証
        for log in data["step_logs"]:
            assert "id" in log
            assert "step_index" in log
            assert "step_name" in log
            assert "action" in log
            assert "status" in log
            assert "started_at" in log
            # action は EXECUTE または COMPENSATE のいずれか
            assert log["action"] in ("EXECUTE", "COMPENSATE")


class TestSagaCancelFlow:
    """Saga のキャンセルフローを検証する。"""

    @pytest.fixture(autouse=True)
    def _skip_if_unavailable(self, saga_client):
        if not _server_available(saga_client.base_url + "/healthz"):
            pytest.skip("Saga server is not running")

    @pytest.fixture(scope="class")
    def registered_workflow(self, saga_client):
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

    def test_cancel_started_saga(self, saga_client, registered_workflow):
        """POST /api/v1/sagas/:id/cancel が CANCELLED または 409 を返す。"""
        start_resp = saga_client.post(
            saga_client.base_url + "/api/v1/sagas",
            json={"workflow_name": registered_workflow},
        )
        if start_resp.status_code == 503:
            pytest.skip("Saga server database is not available")
        assert start_resp.status_code in (200, 201)
        saga_id = start_resp.json()["saga_id"]

        cancel_resp = saga_client.post(
            saga_client.base_url + f"/api/v1/sagas/{saga_id}/cancel",
        )
        assert cancel_resp.status_code in (200, 409, 503)
        if cancel_resp.status_code == 503:
            pytest.skip("Saga server database is not available")
        if cancel_resp.status_code == 200:
            data = cancel_resp.json()
            assert "saga_id" in data
            assert data["status"] == "CANCELLED"

    def test_cancel_nonexistent_saga_returns_404(self, saga_client):
        """POST /api/v1/sagas/:id/cancel に存在しない UUID で 404 を返す。"""
        nonexistent_id = str(uuid.uuid4())
        response = saga_client.post(
            saga_client.base_url + f"/api/v1/sagas/{nonexistent_id}/cancel",
        )
        assert response.status_code in (404, 503)
        if response.status_code == 503:
            pytest.skip("Saga server database is not available")

    def test_get_cancelled_saga_shows_cancelled_status(self, saga_client, registered_workflow):
        """キャンセル後に GET すると status が CANCELLED になる。"""
        start_resp = saga_client.post(
            saga_client.base_url + "/api/v1/sagas",
            json={"workflow_name": registered_workflow},
        )
        if start_resp.status_code == 503:
            pytest.skip("Saga server database is not available")
        assert start_resp.status_code in (200, 201)
        saga_id = start_resp.json()["saga_id"]

        cancel_resp = saga_client.post(
            saga_client.base_url + f"/api/v1/sagas/{saga_id}/cancel",
        )
        if cancel_resp.status_code == 503:
            pytest.skip("Saga server database is not available")
        if cancel_resp.status_code == 409:
            # 既に終了している場合はスキップ
            pytest.skip("Saga already in terminal state, cannot cancel")

        assert cancel_resp.status_code == 200

        detail_resp = saga_client.get(
            saga_client.base_url + f"/api/v1/sagas/{saga_id}",
        )
        assert detail_resp.status_code in (200, 503)
        if detail_resp.status_code == 503:
            pytest.skip("Saga server database is not available")
        data = detail_resp.json()
        assert data["saga"]["status"] == "CANCELLED"
