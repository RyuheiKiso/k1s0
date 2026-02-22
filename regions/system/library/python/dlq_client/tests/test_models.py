"""DLQ クライアントモデルのユニットテスト"""

import uuid

from k1s0_dlq_client.exceptions import DlqClientError, DlqClientErrorCodes
from k1s0_dlq_client.models import DlqMessage, DlqStatus, ListDlqMessagesResponse


def test_dlq_status_values() -> None:
    """DlqStatus の値が正しいこと。"""
    assert DlqStatus.PENDING.value == "PENDING"
    assert DlqStatus.RETRYING.value == "RETRYING"
    assert DlqStatus.RESOLVED.value == "RESOLVED"
    assert DlqStatus.DEAD.value == "DEAD"


def test_dlq_message_from_dict() -> None:
    """辞書から DlqMessage を生成できること。"""
    msg_id = str(uuid.uuid4())
    data = {
        "id": msg_id,
        "original_topic": "events",
        "error_message": "processing failed",
        "retry_count": 2,
        "max_retries": 5,
        "payload": "data",
        "status": "PENDING",
    }
    msg = DlqMessage.from_dict(data)
    assert str(msg.id) == msg_id
    assert msg.original_topic == "events"
    assert msg.retry_count == 2
    assert msg.status == DlqStatus.PENDING


def test_list_dlq_messages_response_from_dict() -> None:
    """辞書から ListDlqMessagesResponse を生成できること。"""
    msg_id = str(uuid.uuid4())
    data = {
        "messages": [
            {
                "id": msg_id,
                "original_topic": "events",
                "error_message": "",
                "retry_count": 0,
                "max_retries": 3,
                "payload": "",
                "status": "PENDING",
            }
        ],
        "total": 1,
        "page": 1,
        "page_size": 20,
    }
    response = ListDlqMessagesResponse.from_dict(data)
    assert response.total == 1
    assert len(response.messages) == 1
    assert response.page == 1


def test_dlq_client_error_with_cause() -> None:
    """DlqClientError に cause を設定できること。"""
    cause = ValueError("root cause")
    error = DlqClientError(
        code=DlqClientErrorCodes.HTTP_ERROR,
        message="something went wrong",
        cause=cause,
    )
    assert error.code == DlqClientErrorCodes.HTTP_ERROR
    assert error.__cause__ is cause


def test_dlq_client_error_str() -> None:
    """DlqClientError の __str__ が 'CODE: message' 形式であること。"""
    error = DlqClientError(
        code=DlqClientErrorCodes.MESSAGE_NOT_FOUND,
        message="not found",
    )
    assert str(error) == "MESSAGE_NOT_FOUND: not found"
