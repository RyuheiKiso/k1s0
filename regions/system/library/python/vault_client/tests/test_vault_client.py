"""vault_client library unit tests."""

import pytest

from k1s0_vault_client import (
    InMemoryVaultClient,
    Secret,
    SecretRotatedEvent,
    VaultClientConfig,
    VaultError,
    VaultErrorCode,
)


def make_config() -> VaultClientConfig:
    return VaultClientConfig(server_url="http://localhost:8080")


def make_secret(path: str) -> Secret:
    return Secret(
        path=path,
        data={"password": "s3cr3t", "username": "admin"},
        version=1,
    )


async def test_get_secret_found() -> None:
    client = InMemoryVaultClient(make_config())
    client.put_secret(make_secret("system/db/primary"))
    secret = await client.get_secret("system/db/primary")
    assert secret.path == "system/db/primary"
    assert secret.data["password"] == "s3cr3t"


async def test_get_secret_not_found() -> None:
    client = InMemoryVaultClient(make_config())
    with pytest.raises(VaultError) as exc_info:
        await client.get_secret("missing/path")
    assert exc_info.value.code == VaultErrorCode.NOT_FOUND


async def test_get_secret_value() -> None:
    client = InMemoryVaultClient(make_config())
    client.put_secret(make_secret("system/db"))
    value = await client.get_secret_value("system/db", "password")
    assert value == "s3cr3t"


async def test_get_secret_value_key_not_found() -> None:
    client = InMemoryVaultClient(make_config())
    client.put_secret(make_secret("system/db"))
    with pytest.raises(VaultError) as exc_info:
        await client.get_secret_value("system/db", "missing_key")
    assert exc_info.value.code == VaultErrorCode.NOT_FOUND


async def test_list_secrets() -> None:
    client = InMemoryVaultClient(make_config())
    client.put_secret(make_secret("system/db/primary"))
    client.put_secret(make_secret("system/db/replica"))
    client.put_secret(make_secret("business/api/key"))
    paths = await client.list_secrets("system/")
    assert len(paths) == 2
    assert all(p.startswith("system/") for p in paths)


async def test_list_secrets_empty() -> None:
    client = InMemoryVaultClient(make_config())
    paths = await client.list_secrets("nothing/")
    assert paths == []


async def test_watch_secret() -> None:
    client = InMemoryVaultClient(make_config())
    watcher = await client.watch_secret("system/db")
    events = [e async for e in watcher]
    assert events == []


async def test_config_defaults() -> None:
    config = VaultClientConfig(server_url="http://vault:8080")
    assert config.cache_ttl == 600.0
    assert config.cache_max_capacity == 500


async def test_config_custom() -> None:
    config = VaultClientConfig(
        server_url="http://vault:8080",
        cache_ttl=300.0,
        cache_max_capacity=100,
    )
    assert config.cache_ttl == 300.0
    assert config.cache_max_capacity == 100


async def test_secret_fields() -> None:
    secret = make_secret("test/path")
    assert secret.path == "test/path"
    assert secret.version == 1
    assert secret.data["username"] == "admin"
    assert secret.created_at is not None


async def test_secret_rotated_event() -> None:
    event = SecretRotatedEvent(path="system/db", version=2)
    assert event.path == "system/db"
    assert event.version == 2


async def test_vault_error_code() -> None:
    err = VaultError(VaultErrorCode.PERMISSION_DENIED, "secret/path")
    assert err.code == VaultErrorCode.PERMISSION_DENIED
    assert "PERMISSION_DENIED" in str(err)


async def test_vault_error_all_codes() -> None:
    for code in VaultErrorCode:
        err = VaultError(code, "test")
        assert err.code == code
