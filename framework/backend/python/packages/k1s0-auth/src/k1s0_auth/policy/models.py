"""Authorization policy data models."""

from __future__ import annotations

from enum import Enum

from pydantic import BaseModel, Field


class Action(str, Enum):
    """Supported authorization actions."""

    READ = "READ"
    WRITE = "WRITE"
    DELETE = "DELETE"
    ADMIN = "ADMIN"


class PolicySubject(BaseModel):
    """The subject requesting access.

    Attributes:
        sub: Subject identifier.
        roles: Assigned roles.
        permissions: Granted permissions.
        groups: Group memberships.
        tenant_id: Optional tenant identifier.
    """

    sub: str
    roles: list[str] = Field(default_factory=list)
    permissions: list[str] = Field(default_factory=list)
    groups: list[str] = Field(default_factory=list)
    tenant_id: str | None = None


class PolicyRequest(BaseModel):
    """An authorization request to evaluate.

    Attributes:
        subject: The requesting subject.
        action: The requested action.
        resource: The target resource identifier.
    """

    subject: PolicySubject
    action: Action
    resource: str


class PolicyRule(BaseModel):
    """A single authorization rule.

    Attributes:
        action: The action this rule applies to.
        resource_pattern: Glob-style resource pattern (e.g. ``orders.*``).
        required_roles: Roles that satisfy this rule (any match).
        required_permissions: Permissions that satisfy this rule (any match).
        allow: Whether matching grants or denies access.
    """

    action: Action
    resource_pattern: str
    required_roles: list[str] = Field(default_factory=list)
    required_permissions: list[str] = Field(default_factory=list)
    allow: bool = True
