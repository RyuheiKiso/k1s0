"""DLQ HTTP クライアント実装"""

from __future__ import annotations

import uuid
from typing import Any

import httpx

from .client import DlqClient
from .exceptions import DlqClientError, DlqClientErrorCodes
from .models import DlqConfig, DlqMessage, ListDlqMessagesResponse, RetryDlqMessageResponse


class HttpDlqClient(DlqClient):
    """httpx を使った DLQ HTTP クライアント。"""

    def __init__(self, config: DlqConfig) -> None:
        self._config = config
        headers: dict[str, str] = {"Content-Type": "application/json"}
        if config.api_key:
            headers["X-API-Key"] = config.api_key
        self._headers = headers

    def _make_client(self) -> httpx.AsyncClient:
        return httpx.AsyncClient(
            base_url=self._config.base_url,
            headers=self._headers,
            timeout=self._config.timeout_seconds,
        )

    def _handle_error(self, resp: httpx.Response, context: str) -> None:
        if resp.status_code == 404:
            raise DlqClientError(
                code=DlqClientErrorCodes.MESSAGE_NOT_FOUND,
                message=f"{context}: message not found",
            )
        if resp.status_code >= 400:
            raise DlqClientError(
                code=DlqClientErrorCodes.HTTP_ERROR,
                message=f"{context}: HTTP {resp.status_code}: {resp.text}",
            )

    async def list_messages(
        self,
        topic: str,
        page: int = 1,
        page_size: int = 20,
    ) -> ListDlqMessagesResponse:
        try:
            async with self._make_client() as client:
                resp = await client.get(
                    "/api/v1/dlq/messages",
                    params={"topic": topic, "page": page, "page_size": page_size},
                )
            self._handle_error(resp, "list_messages")
            data: dict[str, Any] = resp.json()
            return ListDlqMessagesResponse.from_dict(data)
        except DlqClientError:
            raise
        except Exception as e:
            raise DlqClientError(
                code=DlqClientErrorCodes.HTTP_ERROR,
                message=f"Failed to list DLQ messages: {e}",
                cause=e,
            ) from e

    async def get_message(self, message_id: uuid.UUID) -> DlqMessage:
        try:
            async with self._make_client() as client:
                resp = await client.get(f"/api/v1/dlq/messages/{message_id}")
            self._handle_error(resp, f"get_message({message_id})")
            data: dict[str, Any] = resp.json()
            return DlqMessage.from_dict(data)
        except DlqClientError:
            raise
        except Exception as e:
            raise DlqClientError(
                code=DlqClientErrorCodes.HTTP_ERROR,
                message=f"Failed to get DLQ message: {e}",
                cause=e,
            ) from e

    async def retry_message(self, message_id: uuid.UUID) -> RetryDlqMessageResponse:
        try:
            async with self._make_client() as client:
                resp = await client.post(f"/api/v1/dlq/messages/{message_id}/retry")
            self._handle_error(resp, f"retry_message({message_id})")
            data: dict[str, Any] = resp.json()
            return RetryDlqMessageResponse.from_dict(data)
        except DlqClientError:
            raise
        except Exception as e:
            raise DlqClientError(
                code=DlqClientErrorCodes.HTTP_ERROR,
                message=f"Failed to retry DLQ message: {e}",
                cause=e,
            ) from e

    async def delete_message(self, message_id: uuid.UUID) -> None:
        try:
            async with self._make_client() as client:
                resp = await client.delete(f"/api/v1/dlq/messages/{message_id}")
            self._handle_error(resp, f"delete_message({message_id})")
        except DlqClientError:
            raise
        except Exception as e:
            raise DlqClientError(
                code=DlqClientErrorCodes.HTTP_ERROR,
                message=f"Failed to delete DLQ message: {e}",
                cause=e,
            ) from e

    async def retry_all(self, topic: str) -> None:
        try:
            async with self._make_client() as client:
                resp = await client.post(
                    "/api/v1/dlq/messages/retry-all",
                    params={"topic": topic},
                )
            self._handle_error(resp, f"retry_all({topic})")
        except DlqClientError:
            raise
        except Exception as e:
            raise DlqClientError(
                code=DlqClientErrorCodes.HTTP_ERROR,
                message=f"Failed to retry all DLQ messages: {e}",
                cause=e,
            ) from e
