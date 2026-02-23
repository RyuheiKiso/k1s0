"""validation library unit tests."""

import pytest
from k1s0_validation import (
    ValidationError,
    validate_email,
    validate_tenant_id,
    validate_url,
    validate_uuid,
)


def test_validate_email_success() -> None:
    validate_email("user@example.com")


def test_validate_email_failure() -> None:
    with pytest.raises(ValidationError, match="email"):
        validate_email("not-an-email")


def test_validate_email_no_domain() -> None:
    with pytest.raises(ValidationError):
        validate_email("user@")


def test_validate_uuid_success() -> None:
    validate_uuid("550e8400-e29b-41d4-a716-446655440000")


def test_validate_uuid_failure() -> None:
    with pytest.raises(ValidationError, match="id"):
        validate_uuid("not-a-uuid")


def test_validate_uuid_wrong_version() -> None:
    with pytest.raises(ValidationError):
        validate_uuid("550e8400-e29b-31d4-a716-446655440000")


def test_validate_url_success() -> None:
    validate_url("https://example.com/path")


def test_validate_url_failure() -> None:
    with pytest.raises(ValidationError, match="url"):
        validate_url("not-a-url")


def test_validate_url_no_scheme() -> None:
    with pytest.raises(ValidationError):
        validate_url("example.com")


def test_validate_tenant_id_success() -> None:
    validate_tenant_id("my-tenant-01")


def test_validate_tenant_id_failure_uppercase() -> None:
    with pytest.raises(ValidationError, match="tenant_id"):
        validate_tenant_id("My-Tenant")


def test_validate_tenant_id_too_short() -> None:
    with pytest.raises(ValidationError):
        validate_tenant_id("ab")


def test_validate_tenant_id_starts_with_hyphen() -> None:
    with pytest.raises(ValidationError):
        validate_tenant_id("-invalid")
