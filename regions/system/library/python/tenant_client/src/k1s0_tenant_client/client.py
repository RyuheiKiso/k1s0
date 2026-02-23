"""Tenant client for managing tenant information."""

from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from datetime import datetime
from enum import Enum


class TenantStatus(str, Enum):
    """Tenant status types."""

    ACTIVE = "active"
    SUSPENDED = "suspended"
    DELETED = "deleted"


class TenantErrorCode(str, Enum):
    """Tenant error codes."""

    NOT_FOUND = "not_found"
    SUSPENDED = "suspended"
    SERVER_ERROR = "server_error"
    TIMEOUT = "timeout"


class TenantError(Exception):
    """Tenant operation error."""

    def __init__(self, message: str, code: TenantErrorCode) -> None:
        super().__init__(message)
        self.code = code


@dataclass
class Tenant:
    """Tenant information."""

    id: str
    name: str
    status: TenantStatus
    plan: str
    settings: dict[str, str] = field(default_factory=dict)
    created_at: datetime = field(default_factory=datetime.now)


@dataclass
class TenantFilter:
    """Tenant list filter."""

    status: TenantStatus | None = None
    plan: str | None = None


class TenantSettings:
    """Tenant settings wrapper."""

    def __init__(self, values: dict[str, str]) -> None:
        self._values = dict(values)

    @property
    def values(self) -> dict[str, str]:
        """Get all setting values."""
        return dict(self._values)

    def get(self, key: str) -> str | None:
        """Get a setting value by key."""
        return self._values.get(key)


@dataclass
class TenantClientConfig:
    """Tenant client configuration."""

    server_url: str
    cache_ttl: float = 300.0
    cache_max_capacity: int = 1000


class TenantClient(ABC):
    """Abstract tenant client."""

    @abstractmethod
    async def get_tenant(self, tenant_id: str) -> Tenant: ...

    @abstractmethod
    async def list_tenants(self, filter_: TenantFilter | None = None) -> list[Tenant]: ...

    @abstractmethod
    async def is_active(self, tenant_id: str) -> bool: ...

    @abstractmethod
    async def get_settings(self, tenant_id: str) -> TenantSettings: ...


class InMemoryTenantClient(TenantClient):
    """In-memory tenant client for testing."""

    def __init__(self, tenants: list[Tenant] | None = None) -> None:
        self._tenants: list[Tenant] = list(tenants) if tenants else []

    def add_tenant(self, tenant: Tenant) -> None:
        """Add a tenant."""
        self._tenants.append(tenant)

    @property
    def tenants(self) -> list[Tenant]:
        """Get a copy of all tenants."""
        return list(self._tenants)

    async def get_tenant(self, tenant_id: str) -> Tenant:
        for t in self._tenants:
            if t.id == tenant_id:
                return t
        raise TenantError(f"Tenant not found: {tenant_id}", TenantErrorCode.NOT_FOUND)

    async def list_tenants(self, filter_: TenantFilter | None = None) -> list[Tenant]:
        result = self._tenants
        if filter_ is not None:
            if filter_.status is not None:
                result = [t for t in result if t.status == filter_.status]
            if filter_.plan is not None:
                result = [t for t in result if t.plan == filter_.plan]
        return list(result)

    async def is_active(self, tenant_id: str) -> bool:
        tenant = await self.get_tenant(tenant_id)
        return tenant.status == TenantStatus.ACTIVE

    async def get_settings(self, tenant_id: str) -> TenantSettings:
        tenant = await self.get_tenant(tenant_id)
        return TenantSettings(tenant.settings)
