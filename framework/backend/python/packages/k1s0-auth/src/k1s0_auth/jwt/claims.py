"""JWT claims model."""

from __future__ import annotations

from typing import Any

from pydantic import BaseModel, Field


class Claims(BaseModel):
    """Parsed and validated JWT claims.

    Attributes:
        sub: Subject identifier.
        roles: Assigned roles.
        permissions: Granted permissions.
        groups: Group memberships.
        tenant_id: Optional tenant identifier for multi-tenancy.
        custom: Arbitrary additional claims.
    """

    sub: str
    roles: list[str] = Field(default_factory=list)
    permissions: list[str] = Field(default_factory=list)
    groups: list[str] = Field(default_factory=list)
    tenant_id: str | None = None
    custom: dict[str, Any] = Field(default_factory=dict)

    def has_role(self, role: str) -> bool:
        """Check whether the subject has a specific role."""
        return role in self.roles

    def has_permission(self, permission: str) -> bool:
        """Check whether the subject has a specific permission."""
        return permission in self.permissions

    def has_any_role(self, roles: list[str]) -> bool:
        """Check whether the subject has at least one of the given roles."""
        return bool(set(self.roles) & set(roles))
