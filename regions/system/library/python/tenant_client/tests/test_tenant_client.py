"""tenant_client library unit tests."""

import pytest

from k1s0_tenant_client import (
    InMemoryTenantClient,
    Tenant,
    TenantError,
    TenantErrorCode,
    TenantFilter,
    TenantSettings,
    TenantStatus,
)


def make_tenant(
    id: str,
    status: TenantStatus = TenantStatus.ACTIVE,
    plan: str = "basic",
) -> Tenant:
    return Tenant(
        id=id,
        name=f"Tenant {id}",
        status=status,
        plan=plan,
        settings={"max_users": "100"},
    )


async def test_get_tenant() -> None:
    client = InMemoryTenantClient([make_tenant("T-001")])
    tenant = await client.get_tenant("T-001")
    assert tenant.id == "T-001"
    assert tenant.status == TenantStatus.ACTIVE


async def test_get_tenant_not_found() -> None:
    client = InMemoryTenantClient()
    with pytest.raises(TenantError) as exc_info:
        await client.get_tenant("T-999")
    assert exc_info.value.code == TenantErrorCode.NOT_FOUND


async def test_list_tenants_filter_by_status() -> None:
    client = InMemoryTenantClient([
        make_tenant("T-001", TenantStatus.ACTIVE),
        make_tenant("T-002", TenantStatus.SUSPENDED),
        make_tenant("T-003", TenantStatus.ACTIVE),
    ])
    tenants = await client.list_tenants(TenantFilter(status=TenantStatus.ACTIVE))
    assert len(tenants) == 2


async def test_list_tenants_filter_by_plan() -> None:
    client = InMemoryTenantClient([
        make_tenant("T-001", plan="enterprise"),
        make_tenant("T-002", plan="basic"),
    ])
    tenants = await client.list_tenants(TenantFilter(plan="enterprise"))
    assert len(tenants) == 1
    assert tenants[0].id == "T-001"


async def test_list_tenants_no_filter() -> None:
    client = InMemoryTenantClient([make_tenant("T-001"), make_tenant("T-002")])
    tenants = await client.list_tenants()
    assert len(tenants) == 2


async def test_is_active_true() -> None:
    client = InMemoryTenantClient([make_tenant("T-001", TenantStatus.ACTIVE)])
    assert await client.is_active("T-001") is True


async def test_is_active_false() -> None:
    client = InMemoryTenantClient([make_tenant("T-001", TenantStatus.SUSPENDED)])
    assert await client.is_active("T-001") is False


async def test_get_settings() -> None:
    client = InMemoryTenantClient([make_tenant("T-001")])
    settings = await client.get_settings("T-001")
    assert settings.get("max_users") == "100"
    assert settings.get("nonexistent") is None


async def test_add_tenant() -> None:
    client = InMemoryTenantClient()
    client.add_tenant(make_tenant("T-001"))
    tenant = await client.get_tenant("T-001")
    assert tenant.id == "T-001"


async def test_tenant_settings_values() -> None:
    settings = TenantSettings({"key": "value"})
    assert settings.values == {"key": "value"}
    assert settings.get("key") == "value"
    assert settings.get("missing") is None


async def test_tenants_returns_copy() -> None:
    client = InMemoryTenantClient([make_tenant("T-001")])
    t1 = client.tenants
    t2 = client.tenants
    assert t1 is not t2
    assert len(t1) == len(t2)


async def test_tenant_error_properties() -> None:
    error = TenantError("not found", TenantErrorCode.NOT_FOUND)
    assert error.code == TenantErrorCode.NOT_FOUND
    assert str(error) == "not found"
