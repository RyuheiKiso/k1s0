"""Validation rules."""

from __future__ import annotations

import re
from datetime import datetime
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
        raise ValidationError("email", f"Invalid email format: {email}", code="INVALID_EMAIL")


def validate_uuid(id: str) -> None:
    """Validate UUID v4 format."""
    if not _UUID_RE.match(id):
        raise ValidationError("id", f"Invalid UUID v4 format: {id}", code="INVALID_UUID")


def validate_url(url: str) -> None:
    """Validate URL format (must have scheme)."""
    parsed = urlparse(url)
    if not parsed.scheme or not parsed.netloc:
        raise ValidationError("url", f"Invalid URL: {url}", code="INVALID_URL")


def validate_tenant_id(tenant_id: str) -> None:
    """Validate tenant ID format (lowercase alphanumeric with hyphens, 3-63 chars)."""
    if not _TENANT_RE.match(tenant_id):
        raise ValidationError(
            "tenant_id",
            f"Invalid tenant ID: {tenant_id}",
            code="INVALID_TENANT_ID",
        )


def validate_pagination(page: int, per_page: int) -> None:
    """Validate pagination parameters (page >= 1, per_page 1-100)."""
    if page < 1:
        raise ValidationError(
            "page",
            f"Page must be >= 1, got {page}",
            code="INVALID_PAGE",
        )
    if per_page < 1 or per_page > 100:
        raise ValidationError(
            "per_page",
            f"per_page must be 1-100, got {per_page}",
            code="INVALID_PER_PAGE",
        )


def validate_date_range(start_date: datetime, end_date: datetime) -> None:
    """Validate that start_date <= end_date."""
    if start_date > end_date:
        raise ValidationError(
            "date_range",
            f"Start date ({start_date.isoformat()}) must be <= end date ({end_date.isoformat()})",
            code="INVALID_DATE_RANGE",
        )
