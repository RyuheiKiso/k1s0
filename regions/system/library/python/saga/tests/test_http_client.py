"""HttpSagaClient のユニットテスト（respx モック）"""

import httpx
import pytest
import respx
from k1s0_saga.exceptions import SagaError, SagaErrorCodes
from k1s0_saga.http_client import HttpSagaClient
from k1s0_saga.models import SagaConfig, SagaStatus, StartSagaRequest

BASE_URL = "http://saga-server:8080"


def make_client() -> HttpSagaClient:
    return HttpSagaClient(SagaConfig(rest_url=BASE_URL))


@respx.mock
async def test_start_saga_success() -> None:
    """Saga 開始成功。"""
    respx.post(f"{BASE_URL}/api/v1/sagas").mock(
        return_value=httpx.Response(201, json={"saga_id": "saga-123", "status": "STARTED"})
    )
    client = make_client()
    request = StartSagaRequest(workflow_name="order-fulfillment", payload={"order_id": "1"})
    response = await client.start_saga(request)
    assert response.saga_id == "saga-123"
    assert response.status == SagaStatus.STARTED


@respx.mock
async def test_get_saga_success() -> None:
    """Saga 状態取得成功。"""
    respx.get(f"{BASE_URL}/api/v1/sagas/saga-123").mock(
        return_value=httpx.Response(
            200,
            json={
                "saga_id": "saga-123",
                "workflow_name": "order-fulfillment",
                "current_step": "payment",
                "status": "RUNNING",
            },
        )
    )
    client = make_client()
    state = await client.get_saga("saga-123")
    assert state.saga_id == "saga-123"
    assert state.status == SagaStatus.RUNNING


@respx.mock
async def test_get_saga_not_found() -> None:
    """存在しない Saga ID で SagaError(SAGA_NOT_FOUND) が発生すること。"""
    respx.get(f"{BASE_URL}/api/v1/sagas/not-exist").mock(
        return_value=httpx.Response(404, text="Not found")
    )
    client = make_client()
    with pytest.raises(SagaError) as exc_info:
        await client.get_saga("not-exist")
    assert exc_info.value.code == SagaErrorCodes.SAGA_NOT_FOUND


@respx.mock
async def test_cancel_saga_success() -> None:
    """Saga キャンセル成功（例外なし）。"""
    respx.post(f"{BASE_URL}/api/v1/sagas/saga-123/cancel").mock(return_value=httpx.Response(200))
    client = make_client()
    await client.cancel_saga("saga-123")  # Should not raise


@respx.mock
async def test_start_saga_server_error() -> None:
    """サーバーエラー時に SagaError が発生すること。"""
    respx.post(f"{BASE_URL}/api/v1/sagas").mock(
        return_value=httpx.Response(500, text="Internal Server Error")
    )
    client = make_client()
    request = StartSagaRequest(workflow_name="test")
    with pytest.raises(SagaError) as exc_info:
        await client.start_saga(request)
    assert exc_info.value.code == SagaErrorCodes.HTTP_ERROR
