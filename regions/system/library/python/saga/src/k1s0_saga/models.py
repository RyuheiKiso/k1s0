"""Saga データモデル"""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import StrEnum
from typing import Any


class SagaStatus(StrEnum):
    """Saga ステータス。"""

    STARTED = "STARTED"
    RUNNING = "RUNNING"
    COMPLETED = "COMPLETED"
    COMPENSATING = "COMPENSATING"
    FAILED = "FAILED"
    CANCELLED = "CANCELLED"


@dataclass
class SagaStepLog:
    """Saga ステップログ。"""

    step_name: str
    status: str
    started_at: str = ""
    completed_at: str = ""
    error: str = ""

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> SagaStepLog:
        return cls(
            step_name=data.get("step_name", ""),
            status=data.get("status", ""),
            started_at=data.get("started_at", ""),
            completed_at=data.get("completed_at", ""),
            error=data.get("error", ""),
        )


@dataclass
class SagaState:
    """Saga 状態。"""

    saga_id: str
    workflow_name: str
    current_step: str
    status: SagaStatus
    correlation_id: str = ""
    payload: dict[str, Any] = field(default_factory=dict)
    step_logs: list[SagaStepLog] = field(default_factory=list)
    created_at: str = ""
    updated_at: str = ""

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> SagaState:
        return cls(
            saga_id=data["saga_id"],
            workflow_name=data.get("workflow_name", ""),
            current_step=data.get("current_step", ""),
            status=SagaStatus(data.get("status", "STARTED")),
            correlation_id=data.get("correlation_id", ""),
            payload=data.get("payload", {}),
            step_logs=[SagaStepLog.from_dict(s) for s in data.get("step_logs", [])],
            created_at=data.get("created_at", ""),
            updated_at=data.get("updated_at", ""),
        )


@dataclass
class StartSagaRequest:
    """Saga 開始リクエスト。"""

    workflow_name: str
    payload: dict[str, Any] = field(default_factory=dict)
    correlation_id: str = ""

    def to_dict(self) -> dict[str, Any]:
        return {
            "workflow_name": self.workflow_name,
            "payload": self.payload,
            "correlation_id": self.correlation_id,
        }


@dataclass
class StartSagaResponse:
    """Saga 開始レスポンス。"""

    saga_id: str
    status: SagaStatus

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> StartSagaResponse:
        return cls(
            saga_id=data["saga_id"],
            status=SagaStatus(data.get("status", "STARTED")),
        )


@dataclass
class SagaConfig:
    """Saga クライアント設定。"""

    rest_url: str
    timeout_seconds: float = 10.0
    api_key: str = ""
