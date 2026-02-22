"""DLQ サービス CRUD E2E テスト。"""
import uuid

import pytest
import requests


def _server_available(url):
    try:
        requests.get(url, timeout=2)
        return True
    except requests.ConnectionError:
        return False


class TestDlqOperations:
    """DLQ サービスの DLQ 管理エンドポイントを検証する。"""

    @pytest.fixture(autouse=True)
    def _skip_if_unavailable(self, dlq_client):
        if not _server_available(dlq_client.base_url + "/healthz"):
            pytest.skip("DLQ server is not running")

    def test_list_messages_empty_topic(self, dlq_client):
        """GET /api/v1/dlq/test.dlq.v1 が 200 と空リストを返す。"""
        response = dlq_client.get(
            dlq_client.base_url + "/api/v1/dlq/test.dlq.v1",
        )
        assert response.status_code == 200
        data = response.json()
        assert "messages" in data
        assert isinstance(data["messages"], list)
        assert len(data["messages"]) == 0

    def test_get_message_not_found(self, dlq_client):
        """GET /api/v1/dlq/messages/{uuid} が 404 を返す。"""
        nonexistent_id = str(uuid.uuid4())
        response = dlq_client.get(
            dlq_client.base_url + f"/api/v1/dlq/messages/{nonexistent_id}",
        )
        assert response.status_code == 404

    def test_get_message_invalid_id(self, dlq_client):
        """GET /api/v1/dlq/messages/invalid-id が 400 を返す。"""
        response = dlq_client.get(
            dlq_client.base_url + "/api/v1/dlq/messages/invalid-id",
        )
        assert response.status_code == 400

    def test_retry_message_not_found(self, dlq_client):
        """POST /api/v1/dlq/messages/{uuid}/retry が 404 を返す。"""
        nonexistent_id = str(uuid.uuid4())
        response = dlq_client.post(
            dlq_client.base_url + f"/api/v1/dlq/messages/{nonexistent_id}/retry",
        )
        assert response.status_code == 404

    def test_delete_message_not_found(self, dlq_client):
        """DELETE /api/v1/dlq/messages/{uuid} が 404 を返す。"""
        nonexistent_id = str(uuid.uuid4())
        response = dlq_client.delete(
            dlq_client.base_url + f"/api/v1/dlq/messages/{nonexistent_id}",
        )
        assert response.status_code == 404

    def test_retry_all_empty_topic(self, dlq_client):
        """POST /api/v1/dlq/test.dlq.v1/retry-all が 200 と retried=0 を返す。"""
        response = dlq_client.post(
            dlq_client.base_url + "/api/v1/dlq/test.dlq.v1/retry-all",
        )
        assert response.status_code == 200
        data = response.json()
        assert data["retried"] == 0

    def test_list_messages_pagination(self, dlq_client):
        """GET /api/v1/dlq/test.dlq.v1 にページネーションパラメータが効く。"""
        response = dlq_client.get(
            dlq_client.base_url + "/api/v1/dlq/test.dlq.v1",
            params={"page": 1, "page_size": 10},
        )
        assert response.status_code == 200
        data = response.json()
        assert "messages" in data
        assert isinstance(data["messages"], list)
