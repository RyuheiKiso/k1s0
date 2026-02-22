"""Saga モデルのユニットテスト"""

from k1s0_saga.models import SagaState, SagaStatus, StartSagaRequest, StartSagaResponse


def test_saga_status_values() -> None:
    """SagaStatus の値が正しいこと。"""
    assert SagaStatus.STARTED.value == "STARTED"
    assert SagaStatus.RUNNING.value == "RUNNING"
    assert SagaStatus.COMPLETED.value == "COMPLETED"
    assert SagaStatus.COMPENSATING.value == "COMPENSATING"
    assert SagaStatus.FAILED.value == "FAILED"
    assert SagaStatus.CANCELLED.value == "CANCELLED"


def test_start_saga_request_to_dict() -> None:
    """StartSagaRequest が辞書に変換できること。"""
    req = StartSagaRequest(
        workflow_name="order-fulfillment",
        payload={"order_id": "123"},
        correlation_id="corr-abc",
    )
    data = req.to_dict()
    assert data["workflow_name"] == "order-fulfillment"
    assert data["payload"]["order_id"] == "123"
    assert data["correlation_id"] == "corr-abc"


def test_saga_state_from_dict() -> None:
    """辞書から SagaState を生成できること。"""
    data = {
        "saga_id": "saga-123",
        "workflow_name": "order-fulfillment",
        "current_step": "payment",
        "status": "RUNNING",
        "step_logs": [
            {"step_name": "reserve", "status": "COMPLETED"},
        ],
    }
    state = SagaState.from_dict(data)
    assert state.saga_id == "saga-123"
    assert state.status == SagaStatus.RUNNING
    assert len(state.step_logs) == 1
    assert state.step_logs[0].step_name == "reserve"


def test_start_saga_response_from_dict() -> None:
    """辞書から StartSagaResponse を生成できること。"""
    data = {"saga_id": "new-saga-456", "status": "STARTED"}
    resp = StartSagaResponse.from_dict(data)
    assert resp.saga_id == "new-saga-456"
    assert resp.status == SagaStatus.STARTED
