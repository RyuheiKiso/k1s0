"""Validation rules."""

from __future__ import annotations

import re
from urllib.parse import urlparse

from .exceptions import ValidationError

_EMAIL_RE = re.compile(r"^[^@\s]+@[^@\s]+\.[^@\s]+$")
_UUID_RE = re.compile(
    r"^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$",
    re.IGNORECASE,
)
_TENANT_RE = re.compile(r"^[a-z0-9][a-z0-9-]{1,61}[a-z0-9]$")


def validate_email(email: str) -> None:
    """Validate email format."""
    if not _EMAIL_RE.match(email):
        raise ValidationError("email", f"Invalid email format: {email}")


def validate_uuid(id: str) -> None:
    """Validate UUID v4 format."""
    if not _UUID_RE.match(id):
        raise ValidationError("id", f"Invalid UUID v4 format: {id}")


def validate_url(url: str) -> None:
    """Validate URL format (must have scheme)."""
    parsed = urlparse(url)
    if not parsed.scheme or not parsed.netloc:
        raise ValidationError("url", f"Invalid URL: {url}")


def validate_tenant_id(tenant_id: str) -> None:
    """Validate tenant ID format (lowercase alphanumeric with hyphens, 3-63 chars)."""
    if not _TENANT_RE.match(tenant_id):
        raise ValidationError(
            "tenant_id",
            f"Invalid tenant ID: {tenant_id}",
        )
