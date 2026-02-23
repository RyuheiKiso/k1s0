"""DLQ 復旧フロー E2E テスト。

DLQ メッセージのリカバリーフロー（リトライ・一括リトライ・削除）に関する
API の振る舞いを検証する。
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


class TestDlqMessageLifecycle:
    """DLQ メッセージのライフサイクル管理 API を検証する。"""

    @pytest.fixture(autouse=True)
    def _skip_if_unavailable(self, dlq_client):
        if not _server_available(dlq_client.base_url + "/healthz"):
            pytest.skip("DLQ server is not running")

    def test_list_messages_response_structure(self, dlq_client):
        """GET /api/v1/dlq/{topic} のレスポンス構造が仕様通り。"""
        response = dlq_client.get(
            dlq_client.base_url + "/api/v1/dlq/test.dlq.v1",
        )
        assert response.status_code == 200
        data = response.json()
        assert "messages" in data
        assert "pagination" in data
        pagination = data["pagination"]
        assert "total_count" in pagination
        assert "page" in pagination
        assert "page_size" in pagination
        assert "has_next" in pagination
        # デフォルトページネーション値
        assert pagination["page"] == 1
        assert pagination["page_size"] == 20

    def test_list_messages_pagination_parameters(self, dlq_client):
        """GET /api/v1/dlq/{topic} にページネーションパラメータが正しく反映される。"""
        response = dlq_client.get(
            dlq_client.base_url + "/api/v1/dlq/test.dlq.v1",
            params={"page": 2, "page_size": 5},
        )
        assert response.status_code == 200
        data = response.json()
        assert data["pagination"]["page"] == 2
        assert data["pagination"]["page_size"] == 5

    def test_retry_all_empty_topic_returns_zero(self, dlq_client):
        """POST /api/v1/dlq/{topic}/retry-all が空トピックで retried=0 を返す。"""
        response = dlq_client.post(
            dlq_client.base_url + "/api/v1/dlq/empty.nonexistent.topic.v1/retry-all",
        )
        assert response.status_code == 200
        data = response.json()
        assert "retried" in data
        assert "message" in data
        assert data["retried"] == 0

    def test_retry_all_response_structure(self, dlq_client):
        """POST /api/v1/dlq/{topic}/retry-all のレスポンス構造が仕様通り。"""
        response = dlq_client.post(
            dlq_client.base_url + "/api/v1/dlq/test.dlq.v1/retry-all",
        )
        assert response.status_code == 200
        data = response.json()
        assert "retried" in data
        assert isinstance(data["retried"], int)
        assert data["retried"] >= 0
        assert "message" in data
        assert isinstance(data["message"], str)

    def test_multiple_topics_list_independently(self, dlq_client):
        """異なるトピック名での一覧取得が独立して動作する。"""
        topics = ["orders.dlq.v1", "payments.dlq.v1", "inventory.dlq.v1"]
        for topic in topics:
            response = dlq_client.get(
                dlq_client.base_url + f"/api/v1/dlq/{topic}",
            )
            assert response.status_code == 200, (
                f"topic={topic} returned {response.status_code}"
            )
            data = response.json()
            assert "messages" in data
            assert "pagination" in data

    def test_get_message_response_structure_when_found(self, dlq_client):
        """GET /api/v1/dlq/messages/{id} は存在するメッセージの全フィールドを返す。"""
        # まず一覧で実際に存在するメッセージを探す
        list_resp = dlq_client.get(
            dlq_client.base_url + "/api/v1/dlq/test.dlq.v1",
        )
        assert list_resp.status_code == 200
        messages = list_resp.json()["messages"]
        if not messages:
            pytest.skip("No DLQ messages in test.dlq.v1 topic to verify structure")

        msg_id = messages[0]["id"]
        response = dlq_client.get(
            dlq_client.base_url + f"/api/v1/dlq/messages/{msg_id}",
        )
        assert response.status_code == 200
        data = response.json()
        # 必須フィールドの確認
        assert "id" in data
        assert "original_topic" in data
        assert "error_message" in data
        assert "retry_count" in data
        assert "max_retries" in data
        assert "payload" in data
        assert "status" in data
        assert "created_at" in data
        assert "updated_at" in data
        assert "last_retry_at" in data
        # status は有効な値
        assert data["status"] in ("PENDING", "RETRYING", "RESOLVED", "DEAD")


class TestDlqRetryValidation:
    """DLQ リトライのバリデーションを検証する。"""

    @pytest.fixture(autouse=True)
    def _skip_if_unavailable(self, dlq_client):
        if not _server_available(dlq_client.base_url + "/healthz"):
            pytest.skip("DLQ server is not running")

    def test_retry_invalid_uuid_returns_400(self, dlq_client):
        """POST /api/v1/dlq/messages/invalid-id/retry が 400 を返す。"""
        response = dlq_client.post(
            dlq_client.base_url + "/api/v1/dlq/messages/invalid-id/retry",
        )
        assert response.status_code == 400

    def test_retry_nonexistent_message_returns_404(self, dlq_client):
        """POST /api/v1/dlq/messages/{uuid}/retry に存在しない ID で 404 を返す。"""
        nonexistent_id = str(uuid.uuid4())
        response = dlq_client.post(
            dlq_client.base_url + f"/api/v1/dlq/messages/{nonexistent_id}/retry",
        )
        assert response.status_code == 404

    def test_delete_invalid_uuid_returns_400(self, dlq_client):
        """DELETE /api/v1/dlq/messages/invalid-id が 400 を返す。"""
        response = dlq_client.delete(
            dlq_client.base_url + "/api/v1/dlq/messages/invalid-id",
        )
        assert response.status_code == 400

    def test_delete_nonexistent_message_returns_404(self, dlq_client):
        """DELETE /api/v1/dlq/messages/{uuid} に存在しない ID で 404 を返す。"""
        nonexistent_id = str(uuid.uuid4())
        response = dlq_client.delete(
            dlq_client.base_url + f"/api/v1/dlq/messages/{nonexistent_id}",
        )
        assert response.status_code == 404

    def test_get_invalid_uuid_returns_400(self, dlq_client):
        """GET /api/v1/dlq/messages/invalid-id が 400 を返す。"""
        response = dlq_client.get(
            dlq_client.base_url + "/api/v1/dlq/messages/invalid-id",
        )
        assert response.status_code == 400

    def test_get_nonexistent_message_returns_404(self, dlq_client):
        """GET /api/v1/dlq/messages/{uuid} に存在しない ID で 404 を返す。"""
        nonexistent_id = str(uuid.uuid4())
        response = dlq_client.get(
            dlq_client.base_url + f"/api/v1/dlq/messages/{nonexistent_id}",
        )
        assert response.status_code == 404


class TestDlqRecoveryWorkflow:
    """DLQ 復旧ワークフロー（Kafka が動作している環境での統合確認）。"""

    @pytest.fixture(autouse=True)
    def _skip_if_unavailable(self, dlq_client):
        if not _server_available(dlq_client.base_url + "/healthz"):
            pytest.skip("DLQ server is not running")

    def test_retry_all_returns_non_negative_count(self, dlq_client):
        """POST /api/v1/dlq/{topic}/retry-all は常に非負の retried を返す。"""
        for topic in ["orders.dlq.v1", "payments.dlq.v1"]:
            response = dlq_client.post(
                dlq_client.base_url + f"/api/v1/dlq/{topic}/retry-all",
            )
            assert response.status_code == 200
            data = response.json()
            assert data["retried"] >= 0

    def test_retry_flow_for_existing_pending_message(self, dlq_client):
        """PENDING 状態のメッセージが存在する場合のリトライフロー検証。"""
        # test.dlq.v1 から PENDING メッセージを探す
        list_resp = dlq_client.get(
            dlq_client.base_url + "/api/v1/dlq/test.dlq.v1",
        )
        assert list_resp.status_code == 200
        messages = list_resp.json()["messages"]
        pending_msgs = [m for m in messages if m["status"] == "PENDING"]

        if not pending_msgs:
            pytest.skip("No PENDING messages in test.dlq.v1 to test retry flow")

        msg_id = pending_msgs[0]["id"]
        retry_resp = dlq_client.post(
            dlq_client.base_url + f"/api/v1/dlq/messages/{msg_id}/retry",
        )
        # リトライは成功（200）またはリトライ不可（409）のいずれか
        assert retry_resp.status_code in (200, 409)
        if retry_resp.status_code == 200:
            data = retry_resp.json()
            assert "id" in data
            assert "status" in data
            assert data["status"] in ("RETRYING", "RESOLVED")

    def test_delete_existing_message_succeeds(self, dlq_client):
        """存在するメッセージを DELETE するとレスポンスに success=true が含まれる。"""
        list_resp = dlq_client.get(
            dlq_client.base_url + "/api/v1/dlq/test.dlq.v1",
        )
        assert list_resp.status_code == 200
        messages = list_resp.json()["messages"]

        if not messages:
            pytest.skip("No DLQ messages in test.dlq.v1 to test delete")

        msg_id = messages[0]["id"]
        delete_resp = dlq_client.delete(
            dlq_client.base_url + f"/api/v1/dlq/messages/{msg_id}",
        )
        assert delete_resp.status_code == 200
        data = delete_resp.json()
        assert data["success"] is True

        # 削除後は 404 になる
        get_resp = dlq_client.get(
            dlq_client.base_url + f"/api/v1/dlq/messages/{msg_id}",
        )
        assert get_resp.status_code == 404
