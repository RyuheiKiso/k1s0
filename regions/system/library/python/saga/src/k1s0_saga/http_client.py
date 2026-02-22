"""Saga HTTP REST クライアント実装"""

from __future__ import annotations

from typing import Any

import httpx

from .client import SagaClient
from .exceptions import SagaError, SagaErrorCodes
from .models import SagaConfig, SagaState, StartSagaRequest, StartSagaResponse


class HttpSagaClient(SagaClient):
    """httpx を使った Saga HTTP REST クライアント。"""

    def __init__(self, config: SagaConfig) -> None:
        self._config = config
        headers: dict[str, str] = {"Content-Type": "application/json"}
        if config.api_key:
            headers["X-API-Key"] = config.api_key
        self._headers = headers

    def _make_client(self) -> httpx.AsyncClient:
        return httpx.AsyncClient(
            base_url=self._config.rest_url,
            headers=self._headers,
            timeout=self._config.timeout_seconds,
        )

    def _handle_error(self, resp: httpx.Response, context: str) -> None:
        if resp.status_code == 404:
            raise SagaError(
                code=SagaErrorCodes.SAGA_NOT_FOUND,
                message=f"{context}: saga not found",
            )
        if resp.status_code >= 400:
            raise SagaError(
                code=SagaErrorCodes.HTTP_ERROR,
                message=f"{context}: HTTP {resp.status_code}: {resp.text}",
            )

    async def start_saga(self, request: StartSagaRequest) -> StartSagaResponse:
        """Saga を開始する。"""
        try:
            async with self._make_client() as client:
                resp = await client.post("/api/v1/sagas", json=request.to_dict())
            self._handle_error(resp, "start_saga")
            data: dict[str, Any] = resp.json()
            return StartSagaResponse.from_dict(data)
        except SagaError:
            raise
        except Exception as e:
            raise SagaError(
                code=SagaErrorCodes.HTTP_ERROR,
                message=f"Failed to start saga: {e}",
                cause=e,
            ) from e

    async def get_saga(self, saga_id: str) -> SagaState:
        """Saga の状態を取得する。"""
        try:
            async with self._make_client() as client:
                resp = await client.get(f"/api/v1/sagas/{saga_id}")
            self._handle_error(resp, f"get_saga({saga_id})")
            data: dict[str, Any] = resp.json()
            return SagaState.from_dict(data)
        except SagaError:
            raise
        except Exception as e:
            raise SagaError(
                code=SagaErrorCodes.HTTP_ERROR,
                message=f"Failed to get saga: {e}",
                cause=e,
            ) from e

    async def cancel_saga(self, saga_id: str) -> None:
        """Saga をキャンセルする。"""
        try:
            async with self._make_client() as client:
                resp = await client.post(f"/api/v1/sagas/{saga_id}/cancel")
            self._handle_error(resp, f"cancel_saga({saga_id})")
        except SagaError:
            raise
        except Exception as e:
            raise SagaError(
                code=SagaErrorCodes.HTTP_ERROR,
                message=f"Failed to cancel saga: {e}",
                cause=e,
            ) from e
