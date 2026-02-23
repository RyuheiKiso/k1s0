"""Vault client for managing secrets."""

from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from datetime import datetime, timezone
from enum import Enum
from typing import AsyncIterator


class VaultErrorCode(str, Enum):
    """Vault error codes."""

    NOT_FOUND = "NOT_FOUND"
    PERMISSION_DENIED = "PERMISSION_DENIED"
    SERVER_ERROR = "SERVER_ERROR"
    TIMEOUT = "TIMEOUT"
    LEASE_EXPIRED = "LEASE_EXPIRED"


class VaultError(Exception):
    """Vault client error."""

    def __init__(self, code: VaultErrorCode, message: str) -> None:
        super().__init__(f"{code.value}: {message}")
        self.code = code


@dataclass
class Secret:
    """A secret retrieved from the vault."""

    path: str
    data: dict[str, str]
    version: int
    created_at: datetime = field(default_factory=lambda: datetime.now(timezone.utc))


@dataclass
class SecretRotatedEvent:
    """Event emitted when a secret is rotated."""

    path: str
    version: int


@dataclass
class VaultClientConfig:
    """Configuration for the vault client."""

    server_url: str
    cache_ttl: float = 600.0
    cache_max_capacity: int = 500


class VaultClient(ABC):
    """Abstract vault client."""

    @abstractmethod
    async def get_secret(self, path: str) -> Secret: ...

    @abstractmethod
    async def get_secret_value(self, path: str, key: str) -> str: ...

    @abstractmethod
    async def list_secrets(self, path_prefix: str) -> list[str]: ...

    @abstractmethod
    async def watch_secret(self, path: str) -> AsyncIterator[SecretRotatedEvent]: ...


class InMemoryVaultClient(VaultClient):
    """In-memory vault client for testing."""

    def __init__(self, config: VaultClientConfig) -> None:
        self._config = config
        self._store: dict[str, Secret] = {}

    @property
    def config(self) -> VaultClientConfig:
        """Get the client configuration."""
        return self._config

    def put_secret(self, secret: Secret) -> None:
        """Store a secret."""
        self._store[secret.path] = secret

    async def get_secret(self, path: str) -> Secret:
        secret = self._store.get(path)
        if secret is None:
            raise VaultError(VaultErrorCode.NOT_FOUND, path)
        return secret

    async def get_secret_value(self, path: str, key: str) -> str:
        secret = await self.get_secret(path)
        value = secret.data.get(key)
        if value is None:
            raise VaultError(VaultErrorCode.NOT_FOUND, f"{path}/{key}")
        return value

    async def list_secrets(self, path_prefix: str) -> list[str]:
        return [k for k in self._store if k.startswith(path_prefix)]

    async def watch_secret(self, path: str) -> AsyncIterator[SecretRotatedEvent]:
        return _empty_async_iterator()


# Alias for design doc compatibility
GrpcVaultClient = InMemoryVaultClient


async def _empty_async_iterator() -> AsyncIterator[SecretRotatedEvent]:
    return
    yield  # type: ignore[misc]
