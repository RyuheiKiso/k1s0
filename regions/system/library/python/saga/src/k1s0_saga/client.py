"""SagaClient 抽象基底クラス"""

from __future__ import annotations

from abc import ABC, abstractmethod

from .models import SagaState, StartSagaRequest, StartSagaResponse


class SagaClient(ABC):
    """Saga クライアント抽象基底クラス。"""

    @abstractmethod
    async def start_saga(self, request: StartSagaRequest) -> StartSagaResponse:
        """Saga を開始する。"""
        ...

    @abstractmethod
    async def get_saga(self, saga_id: str) -> SagaState:
        """Saga の状態を取得する。"""
        ...

    @abstractmethod
    async def cancel_saga(self, saga_id: str) -> None:
        """Saga をキャンセルする。"""
        ...
