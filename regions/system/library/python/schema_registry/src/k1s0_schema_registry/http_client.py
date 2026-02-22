"""Schema Registry HTTP クライアント実装"""

from __future__ import annotations

from typing import Any

import httpx

from .client import SchemaRegistryClient
from .exceptions import SchemaRegistryError, SchemaRegistryErrorCodes
from .models import RegisteredSchema, SchemaRegistryConfig, SchemaType


class HttpSchemaRegistryClient(SchemaRegistryClient):
    """httpx を使った Schema Registry HTTP クライアント。"""

    def __init__(self, config: SchemaRegistryConfig) -> None:
        self._config = config
        self._auth: tuple[str, str] | None = None
        if config.username and config.password:
            self._auth = (config.username, config.password)

    def _make_sync_client(self) -> httpx.Client:
        return httpx.Client(
            base_url=self._config.url,
            auth=self._auth,
            timeout=self._config.timeout_seconds,
        )

    def _make_async_client(self) -> httpx.AsyncClient:
        return httpx.AsyncClient(
            base_url=self._config.url,
            auth=self._auth,
            timeout=self._config.timeout_seconds,
        )

    def _handle_error(self, resp: httpx.Response, context: str) -> None:
        if resp.status_code == 404:
            raise SchemaRegistryError(
                code=SchemaRegistryErrorCodes.SCHEMA_NOT_FOUND,
                message=f"{context}: schema not found",
            )
        if resp.status_code >= 400:
            raise SchemaRegistryError(
                code=SchemaRegistryErrorCodes.HTTP_ERROR,
                message=f"{context}: HTTP {resp.status_code}: {resp.text}",
            )

    def register_schema(
        self,
        subject: str,
        schema: str,
        schema_type: SchemaType = SchemaType.AVRO,
    ) -> int:
        body: dict[str, Any] = {"schema": schema, "schemaType": schema_type.value}
        try:
            with self._make_sync_client() as client:
                resp = client.post(f"/subjects/{subject}/versions", json=body)
            self._handle_error(resp, f"register_schema({subject})")
            result: dict[str, Any] = resp.json()
            return int(result["id"])
        except SchemaRegistryError:
            raise
        except Exception as e:
            raise SchemaRegistryError(
                code=SchemaRegistryErrorCodes.HTTP_ERROR,
                message=f"Failed to register schema: {e}",
                cause=e,
            ) from e

    async def register_schema_async(
        self,
        subject: str,
        schema: str,
        schema_type: SchemaType = SchemaType.AVRO,
    ) -> int:
        body: dict[str, Any] = {"schema": schema, "schemaType": schema_type.value}
        try:
            async with self._make_async_client() as client:
                resp = await client.post(f"/subjects/{subject}/versions", json=body)
            self._handle_error(resp, f"register_schema_async({subject})")
            result: dict[str, Any] = resp.json()
            return int(result["id"])
        except SchemaRegistryError:
            raise
        except Exception as e:
            raise SchemaRegistryError(
                code=SchemaRegistryErrorCodes.HTTP_ERROR,
                message=f"Failed to register schema async: {e}",
                cause=e,
            ) from e

    def get_schema_by_id(self, schema_id: int) -> RegisteredSchema:
        try:
            with self._make_sync_client() as client:
                resp = client.get(f"/schemas/ids/{schema_id}")
            self._handle_error(resp, f"get_schema_by_id({schema_id})")
            data: dict[str, Any] = resp.json()
            schema_type_str = data.get("schemaType", "AVRO")
            return RegisteredSchema(
                id=schema_id,
                subject="",
                version=0,
                schema=data["schema"],
                schema_type=SchemaType(schema_type_str),
            )
        except SchemaRegistryError:
            raise
        except Exception as e:
            raise SchemaRegistryError(
                code=SchemaRegistryErrorCodes.HTTP_ERROR,
                message=f"Failed to get schema: {e}",
                cause=e,
            ) from e

    async def get_schema_by_id_async(self, schema_id: int) -> RegisteredSchema:
        try:
            async with self._make_async_client() as client:
                resp = await client.get(f"/schemas/ids/{schema_id}")
            self._handle_error(resp, f"get_schema_by_id_async({schema_id})")
            data: dict[str, Any] = resp.json()
            schema_type_str = data.get("schemaType", "AVRO")
            return RegisteredSchema(
                id=schema_id,
                subject="",
                version=0,
                schema=data["schema"],
                schema_type=SchemaType(schema_type_str),
            )
        except SchemaRegistryError:
            raise
        except Exception as e:
            raise SchemaRegistryError(
                code=SchemaRegistryErrorCodes.HTTP_ERROR,
                message=f"Failed to get schema async: {e}",
                cause=e,
            ) from e

    def check_compatibility(self, subject: str, schema: str) -> bool:
        body: dict[str, Any] = {"schema": schema}
        try:
            with self._make_sync_client() as client:
                resp = client.post(
                    f"/compatibility/subjects/{subject}/versions/latest",
                    json=body,
                )
            if resp.status_code == 404:
                return True  # subject が存在しない場合は互換性あり
            self._handle_error(resp, f"check_compatibility({subject})")
            result: dict[str, Any] = resp.json()
            is_compat: bool = result.get("is_compatible", False)
            return is_compat
        except SchemaRegistryError:
            raise
        except Exception as e:
            raise SchemaRegistryError(
                code=SchemaRegistryErrorCodes.HTTP_ERROR,
                message=f"Failed to check compatibility: {e}",
                cause=e,
            ) from e

    async def check_compatibility_async(self, subject: str, schema: str) -> bool:
        body: dict[str, Any] = {"schema": schema}
        try:
            async with self._make_async_client() as client:
                resp = await client.post(
                    f"/compatibility/subjects/{subject}/versions/latest",
                    json=body,
                )
            if resp.status_code == 404:
                return True
            self._handle_error(resp, f"check_compatibility_async({subject})")
            result: dict[str, Any] = resp.json()
            is_compat: bool = result.get("is_compatible", False)
            return is_compat
        except SchemaRegistryError:
            raise
        except Exception as e:
            raise SchemaRegistryError(
                code=SchemaRegistryErrorCodes.HTTP_ERROR,
                message=f"Failed to check compatibility async: {e}",
                cause=e,
            ) from e
