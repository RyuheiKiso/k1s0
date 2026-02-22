"""HttpSchemaRegistryClient のユニットテスト（respx モック）"""

import httpx
import pytest
import respx
from k1s0_schema_registry.exceptions import SchemaRegistryError, SchemaRegistryErrorCodes
from k1s0_schema_registry.http_client import HttpSchemaRegistryClient
from k1s0_schema_registry.models import SchemaRegistryConfig, SchemaType

BASE_URL = "http://schema-registry:8081"


def make_client() -> HttpSchemaRegistryClient:
    return HttpSchemaRegistryClient(SchemaRegistryConfig(url=BASE_URL))


@respx.mock
def test_register_schema_success() -> None:
    """スキーマ登録成功時に ID が返ること。"""
    respx.post(f"{BASE_URL}/subjects/test-value/versions").mock(
        return_value=httpx.Response(200, json={"id": 42})
    )
    client = make_client()
    schema_id = client.register_schema("test-value", '{"type":"string"}')
    assert schema_id == 42


@respx.mock
def test_register_schema_error() -> None:
    """スキーマ登録失敗時に SchemaRegistryError が発生すること。"""
    respx.post(f"{BASE_URL}/subjects/test-value/versions").mock(
        return_value=httpx.Response(500, text="Internal Server Error")
    )
    client = make_client()
    with pytest.raises(SchemaRegistryError) as exc_info:
        client.register_schema("test-value", '{"type":"string"}')
    assert exc_info.value.code == SchemaRegistryErrorCodes.HTTP_ERROR


@respx.mock
def test_get_schema_by_id_success() -> None:
    """ID によるスキーマ取得成功。"""
    respx.get(f"{BASE_URL}/schemas/ids/1").mock(
        return_value=httpx.Response(200, json={"schema": '{"type":"string"}', "schemaType": "AVRO"})
    )
    client = make_client()
    schema = client.get_schema_by_id(1)
    assert schema.schema == '{"type":"string"}'
    assert schema.schema_type == SchemaType.AVRO


@respx.mock
def test_get_schema_by_id_not_found() -> None:
    """存在しない ID で SchemaRegistryError(SCHEMA_NOT_FOUND) が発生すること。"""
    respx.get(f"{BASE_URL}/schemas/ids/999").mock(
        return_value=httpx.Response(404, text="Not found")
    )
    client = make_client()
    with pytest.raises(SchemaRegistryError) as exc_info:
        client.get_schema_by_id(999)
    assert exc_info.value.code == SchemaRegistryErrorCodes.SCHEMA_NOT_FOUND


@respx.mock
def test_check_compatibility_compatible() -> None:
    """互換性あり時に True が返ること。"""
    respx.post(f"{BASE_URL}/compatibility/subjects/test-value/versions/latest").mock(
        return_value=httpx.Response(200, json={"is_compatible": True})
    )
    client = make_client()
    result = client.check_compatibility("test-value", '{"type":"string"}')
    assert result is True


@respx.mock
def test_check_compatibility_not_found_returns_true() -> None:
    """subject が存在しない場合は True（互換性あり）が返ること。"""
    respx.post(f"{BASE_URL}/compatibility/subjects/new-subject/versions/latest").mock(
        return_value=httpx.Response(404, text="Not found")
    )
    client = make_client()
    result = client.check_compatibility("new-subject", '{"type":"string"}')
    assert result is True


@respx.mock
async def test_register_schema_async_success() -> None:
    """非同期スキーマ登録成功。"""
    respx.post(f"{BASE_URL}/subjects/test-value/versions").mock(
        return_value=httpx.Response(200, json={"id": 99})
    )
    client = make_client()
    schema_id = await client.register_schema_async("test-value", '{"type":"string"}')
    assert schema_id == 99


@respx.mock
async def test_register_schema_async_error() -> None:
    """非同期スキーマ登録失敗時に SchemaRegistryError が発生すること。"""
    respx.post(f"{BASE_URL}/subjects/test-value/versions").mock(
        return_value=httpx.Response(500, text="Internal Server Error")
    )
    client = make_client()
    with pytest.raises(SchemaRegistryError) as exc_info:
        await client.register_schema_async("test-value", '{"type":"string"}')
    assert exc_info.value.code == SchemaRegistryErrorCodes.HTTP_ERROR


@respx.mock
async def test_get_schema_by_id_async_success() -> None:
    """非同期 ID によるスキーマ取得成功。"""
    respx.get(f"{BASE_URL}/schemas/ids/5").mock(
        return_value=httpx.Response(200, json={"schema": '{"type":"int"}', "schemaType": "AVRO"})
    )
    client = make_client()
    schema = await client.get_schema_by_id_async(5)
    assert schema.schema == '{"type":"int"}'
    assert schema.schema_type == SchemaType.AVRO


@respx.mock
async def test_get_schema_by_id_async_not_found() -> None:
    """非同期で存在しない ID で SchemaRegistryError(SCHEMA_NOT_FOUND) が発生すること。"""
    respx.get(f"{BASE_URL}/schemas/ids/999").mock(
        return_value=httpx.Response(404, text="Not found")
    )
    client = make_client()
    with pytest.raises(SchemaRegistryError) as exc_info:
        await client.get_schema_by_id_async(999)
    assert exc_info.value.code == SchemaRegistryErrorCodes.SCHEMA_NOT_FOUND


@respx.mock
async def test_check_compatibility_async_compatible() -> None:
    """非同期互換性あり時に True が返ること。"""
    respx.post(f"{BASE_URL}/compatibility/subjects/test-value/versions/latest").mock(
        return_value=httpx.Response(200, json={"is_compatible": True})
    )
    client = make_client()
    result = await client.check_compatibility_async("test-value", '{"type":"string"}')
    assert result is True


@respx.mock
async def test_check_compatibility_async_not_found_returns_true() -> None:
    """非同期で subject が存在しない場合は True が返ること。"""
    respx.post(f"{BASE_URL}/compatibility/subjects/new-subject/versions/latest").mock(
        return_value=httpx.Response(404, text="Not found")
    )
    client = make_client()
    result = await client.check_compatibility_async("new-subject", '{"type":"string"}')
    assert result is True


@respx.mock
async def test_check_compatibility_async_error() -> None:
    """非同期互換性チェック失敗時に SchemaRegistryError が発生すること。"""
    respx.post(f"{BASE_URL}/compatibility/subjects/test-value/versions/latest").mock(
        return_value=httpx.Response(500, text="Server Error")
    )
    client = make_client()
    with pytest.raises(SchemaRegistryError) as exc_info:
        await client.check_compatibility_async("test-value", '{"type":"string"}')
    assert exc_info.value.code == SchemaRegistryErrorCodes.HTTP_ERROR


@respx.mock
async def test_get_schema_by_id_async_error() -> None:
    """非同期スキーマ取得でサーバーエラー時に SchemaRegistryError が発生すること。"""
    respx.get(f"{BASE_URL}/schemas/ids/10").mock(
        return_value=httpx.Response(503, text="Service Unavailable")
    )
    client = make_client()
    with pytest.raises(SchemaRegistryError) as exc_info:
        await client.get_schema_by_id_async(10)
    assert exc_info.value.code == SchemaRegistryErrorCodes.HTTP_ERROR


@respx.mock
def test_check_compatibility_error() -> None:
    """互換性チェック失敗時に SchemaRegistryError が発生すること。"""
    respx.post(f"{BASE_URL}/compatibility/subjects/test-value/versions/latest").mock(
        return_value=httpx.Response(500, text="Server Error")
    )
    client = make_client()
    with pytest.raises(SchemaRegistryError) as exc_info:
        client.check_compatibility("test-value", '{"type":"string"}')
    assert exc_info.value.code == SchemaRegistryErrorCodes.HTTP_ERROR


@respx.mock
def test_get_schema_by_id_error() -> None:
    """スキーマ取得でサーバーエラー時に SchemaRegistryError が発生すること。"""
    respx.get(f"{BASE_URL}/schemas/ids/10").mock(
        return_value=httpx.Response(503, text="Service Unavailable")
    )
    client = make_client()
    with pytest.raises(SchemaRegistryError) as exc_info:
        client.get_schema_by_id(10)
    assert exc_info.value.code == SchemaRegistryErrorCodes.HTTP_ERROR


def test_client_with_auth() -> None:
    """username/password 指定時に認証が設定されること。"""
    config = SchemaRegistryConfig(
        url=BASE_URL,
        username="user",
        password="pass",
    )
    client = HttpSchemaRegistryClient(config)
    assert client._auth == ("user", "pass")


def test_client_without_auth() -> None:
    """username/password 未指定時に認証が None であること。"""
    client = make_client()
    assert client._auth is None


def test_schema_registry_error_str() -> None:
    """SchemaRegistryError の __str__ がコードを含むこと。"""
    err = SchemaRegistryError(code="TEST_CODE", message="test message")
    assert "TEST_CODE" in str(err)
    assert "test message" in str(err)


def test_schema_registry_error_with_cause() -> None:
    """SchemaRegistryError に cause が設定されること。"""
    cause = ValueError("original error")
    err = SchemaRegistryError(code="HTTP_ERROR", message="wrapped", cause=cause)
    assert err.__cause__ is cause


@respx.mock
def test_register_schema_network_error() -> None:
    """ネットワークエラー時に SchemaRegistryError が発生すること。"""
    respx.post(f"{BASE_URL}/subjects/test-value/versions").mock(
        side_effect=httpx.ConnectError("connection refused")
    )
    client = make_client()
    with pytest.raises(SchemaRegistryError) as exc_info:
        client.register_schema("test-value", '{"type":"string"}')
    assert exc_info.value.code == SchemaRegistryErrorCodes.HTTP_ERROR


@respx.mock
async def test_register_schema_async_network_error() -> None:
    """非同期ネットワークエラー時に SchemaRegistryError が発生すること。"""
    respx.post(f"{BASE_URL}/subjects/test-value/versions").mock(
        side_effect=httpx.ConnectError("connection refused")
    )
    client = make_client()
    with pytest.raises(SchemaRegistryError) as exc_info:
        await client.register_schema_async("test-value", '{"type":"string"}')
    assert exc_info.value.code == SchemaRegistryErrorCodes.HTTP_ERROR


@respx.mock
async def test_get_schema_by_id_async_network_error() -> None:
    """非同期スキーマ取得でネットワークエラー時に SchemaRegistryError が発生すること。"""
    respx.get(f"{BASE_URL}/schemas/ids/1").mock(
        side_effect=httpx.ConnectError("connection refused")
    )
    client = make_client()
    with pytest.raises(SchemaRegistryError) as exc_info:
        await client.get_schema_by_id_async(1)
    assert exc_info.value.code == SchemaRegistryErrorCodes.HTTP_ERROR


@respx.mock
async def test_check_compatibility_async_network_error() -> None:
    """非同期互換性チェックでネットワークエラー時に SchemaRegistryError が発生すること。"""
    respx.post(f"{BASE_URL}/compatibility/subjects/test-value/versions/latest").mock(
        side_effect=httpx.ConnectError("connection refused")
    )
    client = make_client()
    with pytest.raises(SchemaRegistryError) as exc_info:
        await client.check_compatibility_async("test-value", '{"type":"string"}')
    assert exc_info.value.code == SchemaRegistryErrorCodes.HTTP_ERROR


@respx.mock
def test_check_compatibility_network_error() -> None:
    """互換性チェックでネットワークエラー時に SchemaRegistryError が発生すること。"""
    respx.post(f"{BASE_URL}/compatibility/subjects/test-value/versions/latest").mock(
        side_effect=httpx.ConnectError("connection refused")
    )
    client = make_client()
    with pytest.raises(SchemaRegistryError) as exc_info:
        client.check_compatibility("test-value", '{"type":"string"}')
    assert exc_info.value.code == SchemaRegistryErrorCodes.HTTP_ERROR


@respx.mock
def test_get_schema_by_id_network_error() -> None:
    """スキーマ取得でネットワークエラー時に SchemaRegistryError が発生すること。"""
    respx.get(f"{BASE_URL}/schemas/ids/1").mock(
        side_effect=httpx.ConnectError("connection refused")
    )
    client = make_client()
    with pytest.raises(SchemaRegistryError) as exc_info:
        client.get_schema_by_id(1)
    assert exc_info.value.code == SchemaRegistryErrorCodes.HTTP_ERROR
