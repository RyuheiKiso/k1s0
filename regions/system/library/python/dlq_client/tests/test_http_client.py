"""HttpDlqClient のユニットテスト（respx モック）"""

import uuid

import httpx
import pytest
import respx
from k1s0_dlq_client.exceptions import DlqClientError, DlqClientErrorCodes
from k1s0_dlq_client.http_client import HttpDlqClient
from k1s0_dlq_client.models import DlqConfig, DlqStatus

BASE_URL = "http://dlq-server:8080"


def make_client() -> HttpDlqClient:
    return HttpDlqClient(DlqConfig(base_url=BASE_URL))


def make_client_with_api_key() -> HttpDlqClient:
    return HttpDlqClient(DlqConfig(base_url=BASE_URL, api_key="test-key"))


@respx.mock
async def test_list_messages_success() -> None:
    """メッセージ一覧取得成功。"""
    msg_id = str(uuid.uuid4())
    respx.get(f"{BASE_URL}/api/v1/dlq/messages").mock(
        return_value=httpx.Response(
            200,
            json={
                "messages": [
                    {
                        "id": msg_id,
                        "original_topic": "events",
                        "error_message": "error",
                        "retry_count": 1,
                        "max_retries": 3,
                        "payload": "",
                        "status": "PENDING",
                    }
                ],
                "total": 1,
                "page": 1,
                "page_size": 20,
            },
        )
    )
    client = make_client()
    response = await client.list_messages("events")
    assert response.total == 1
    assert len(response.messages) == 1


@respx.mock
async def test_get_message_not_found() -> None:
    """存在しない ID で DlqClientError(MESSAGE_NOT_FOUND) が発生すること。"""
    msg_id = uuid.uuid4()
    respx.get(f"{BASE_URL}/api/v1/dlq/messages/{msg_id}").mock(
        return_value=httpx.Response(404, text="Not found")
    )
    client = make_client()
    with pytest.raises(DlqClientError) as exc_info:
        await client.get_message(msg_id)
    assert exc_info.value.code == DlqClientErrorCodes.MESSAGE_NOT_FOUND


@respx.mock
async def test_retry_message_success() -> None:
    """メッセージリトライ成功。"""
    msg_id = uuid.uuid4()
    respx.post(f"{BASE_URL}/api/v1/dlq/messages/{msg_id}/retry").mock(
        return_value=httpx.Response(
            200,
            json={
                "message_id": str(msg_id),
                "status": "RETRYING",
                "message": "Retry scheduled",
            },
        )
    )
    client = make_client()
    response = await client.retry_message(msg_id)
    assert response.status == DlqStatus.RETRYING


@respx.mock
async def test_delete_message_success() -> None:
    """メッセージ削除成功（例外なし）。"""
    msg_id = uuid.uuid4()
    respx.delete(f"{BASE_URL}/api/v1/dlq/messages/{msg_id}").mock(return_value=httpx.Response(204))
    client = make_client()
    await client.delete_message(msg_id)  # Should not raise


@respx.mock
async def test_retry_all_success() -> None:
    """全メッセージリトライ成功（例外なし）。"""
    respx.post(f"{BASE_URL}/api/v1/dlq/messages/retry-all").mock(return_value=httpx.Response(200))
    client = make_client()
    await client.retry_all("events")  # Should not raise


@respx.mock
async def test_client_with_api_key_sets_header() -> None:
    """api_key が設定された場合に X-API-Key ヘッダーが付与されること。"""
    str(uuid.uuid4())
    respx.get(f"{BASE_URL}/api/v1/dlq/messages").mock(
        return_value=httpx.Response(
            200,
            json={
                "messages": [],
                "total": 0,
                "page": 1,
                "page_size": 20,
            },
        )
    )
    client = make_client_with_api_key()
    response = await client.list_messages("events")
    assert response.total == 0


@respx.mock
async def test_list_messages_http_error() -> None:
    """list_messages で 500 エラーが発生した場合に DlqClientError(HTTP_ERROR) になること。"""
    respx.get(f"{BASE_URL}/api/v1/dlq/messages").mock(
        return_value=httpx.Response(500, text="Internal Server Error")
    )
    client = make_client()
    with pytest.raises(DlqClientError) as exc_info:
        await client.list_messages("events")
    assert exc_info.value.code == DlqClientErrorCodes.HTTP_ERROR


@respx.mock
async def test_get_message_success() -> None:
    """get_message 成功。"""
    msg_id = uuid.uuid4()
    respx.get(f"{BASE_URL}/api/v1/dlq/messages/{msg_id}").mock(
        return_value=httpx.Response(
            200,
            json={
                "id": str(msg_id),
                "original_topic": "events",
                "error_message": "error",
                "retry_count": 1,
                "max_retries": 3,
                "payload": "",
                "status": "PENDING",
            },
        )
    )
    client = make_client()
    msg = await client.get_message(msg_id)
    assert msg.id == msg_id
    assert msg.status == DlqStatus.PENDING


@respx.mock
async def test_get_message_http_error() -> None:
    """get_message で 500 エラーが発生した場合に DlqClientError(HTTP_ERROR) になること。"""
    msg_id = uuid.uuid4()
    respx.get(f"{BASE_URL}/api/v1/dlq/messages/{msg_id}").mock(
        return_value=httpx.Response(500, text="Internal Server Error")
    )
    client = make_client()
    with pytest.raises(DlqClientError) as exc_info:
        await client.get_message(msg_id)
    assert exc_info.value.code == DlqClientErrorCodes.HTTP_ERROR


@respx.mock
async def test_retry_message_http_error() -> None:
    """retry_message で 500 エラーが発生した場合に DlqClientError(HTTP_ERROR) になること。"""
    msg_id = uuid.uuid4()
    respx.post(f"{BASE_URL}/api/v1/dlq/messages/{msg_id}/retry").mock(
        return_value=httpx.Response(500, text="Internal Server Error")
    )
    client = make_client()
    with pytest.raises(DlqClientError) as exc_info:
        await client.retry_message(msg_id)
    assert exc_info.value.code == DlqClientErrorCodes.HTTP_ERROR


@respx.mock
async def test_delete_message_http_error() -> None:
    """delete_message で 500 エラーが発生した場合に DlqClientError(HTTP_ERROR) になること。"""
    msg_id = uuid.uuid4()
    respx.delete(f"{BASE_URL}/api/v1/dlq/messages/{msg_id}").mock(
        return_value=httpx.Response(500, text="Internal Server Error")
    )
    client = make_client()
    with pytest.raises(DlqClientError) as exc_info:
        await client.delete_message(msg_id)
    assert exc_info.value.code == DlqClientErrorCodes.HTTP_ERROR


@respx.mock
async def test_retry_all_http_error() -> None:
    """retry_all で 500 エラーが発生した場合に DlqClientError(HTTP_ERROR) になること。"""
    respx.post(f"{BASE_URL}/api/v1/dlq/messages/retry-all").mock(
        return_value=httpx.Response(500, text="Internal Server Error")
    )
    client = make_client()
    with pytest.raises(DlqClientError) as exc_info:
        await client.retry_all("events")
    assert exc_info.value.code == DlqClientErrorCodes.HTTP_ERROR


async def test_list_messages_network_error() -> None:
    """ネットワークエラーの場合に DlqClientError(HTTP_ERROR) になること。"""
    with respx.mock:
        respx.get(f"{BASE_URL}/api/v1/dlq/messages").mock(
            side_effect=httpx.ConnectError("Connection refused")
        )
        client = make_client()
        with pytest.raises(DlqClientError) as exc_info:
            await client.list_messages("events")
        assert exc_info.value.code == DlqClientErrorCodes.HTTP_ERROR


async def test_get_message_network_error() -> None:
    """get_message のネットワークエラーの場合に DlqClientError(HTTP_ERROR) になること。"""
    msg_id = uuid.uuid4()
    with respx.mock:
        respx.get(f"{BASE_URL}/api/v1/dlq/messages/{msg_id}").mock(
            side_effect=httpx.ConnectError("Connection refused")
        )
        client = make_client()
        with pytest.raises(DlqClientError) as exc_info:
            await client.get_message(msg_id)
        assert exc_info.value.code == DlqClientErrorCodes.HTTP_ERROR


async def test_retry_message_network_error() -> None:
    """retry_message のネットワークエラーの場合に DlqClientError(HTTP_ERROR) になること。"""
    msg_id = uuid.uuid4()
    with respx.mock:
        respx.post(f"{BASE_URL}/api/v1/dlq/messages/{msg_id}/retry").mock(
            side_effect=httpx.ConnectError("Connection refused")
        )
        client = make_client()
        with pytest.raises(DlqClientError) as exc_info:
            await client.retry_message(msg_id)
        assert exc_info.value.code == DlqClientErrorCodes.HTTP_ERROR


async def test_delete_message_network_error() -> None:
    """delete_message のネットワークエラーの場合に DlqClientError(HTTP_ERROR) になること。"""
    msg_id = uuid.uuid4()
    with respx.mock:
        respx.delete(f"{BASE_URL}/api/v1/dlq/messages/{msg_id}").mock(
            side_effect=httpx.ConnectError("Connection refused")
        )
        client = make_client()
        with pytest.raises(DlqClientError) as exc_info:
            await client.delete_message(msg_id)
        assert exc_info.value.code == DlqClientErrorCodes.HTTP_ERROR


async def test_retry_all_network_error() -> None:
    """retry_all のネットワークエラーの場合に DlqClientError(HTTP_ERROR) になること。"""
    with respx.mock:
        respx.post(f"{BASE_URL}/api/v1/dlq/messages/retry-all").mock(
            side_effect=httpx.ConnectError("Connection refused")
        )
        client = make_client()
        with pytest.raises(DlqClientError) as exc_info:
            await client.retry_all("events")
        assert exc_info.value.code == DlqClientErrorCodes.HTTP_ERROR
