"""Session models."""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime


@dataclass
class Session:
    """Session data."""

    id: str
    user_id: str
    token: str
    expires_at: datetime
    created_at: datetime
    revoked: bool = False
    metadata: dict[str, str] = field(default_factory=dict)


@dataclass
class CreateSessionRequest:
    """Create session request."""

    user_id: str
    ttl_seconds: int
    metadata: dict[str, str] = field(default_factory=dict)


@dataclass
class RefreshSessionRequest:
    """Refresh session request."""

    id: str
    ttl_seconds: int
