"""validation library unit tests."""

from datetime import datetime

import pytest
from k1s0_validation import (
    ValidationError,
    ValidationErrors,
    validate_date_range,
    validate_email,
    validate_pagination,
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


# --- validate_pagination tests ---


def test_validate_pagination_success() -> None:
    validate_pagination(1, 10)
    validate_pagination(1, 1)
    validate_pagination(1, 100)
    validate_pagination(999, 50)


def test_validate_pagination_page_less_than_1() -> None:
    with pytest.raises(ValidationError) as exc_info:
        validate_pagination(0, 10)
    assert exc_info.value.code == "INVALID_PAGE"


def test_validate_pagination_negative_page() -> None:
    with pytest.raises(ValidationError) as exc_info:
        validate_pagination(-1, 10)
    assert exc_info.value.code == "INVALID_PAGE"


def test_validate_pagination_per_page_too_low() -> None:
    with pytest.raises(ValidationError) as exc_info:
        validate_pagination(1, 0)
    assert exc_info.value.code == "INVALID_PER_PAGE"


def test_validate_pagination_per_page_too_high() -> None:
    with pytest.raises(ValidationError) as exc_info:
        validate_pagination(1, 101)
    assert exc_info.value.code == "INVALID_PER_PAGE"


# --- validate_date_range tests ---


def test_validate_date_range_success() -> None:
    start = datetime(2024, 1, 1, 0, 0, 0)
    end = datetime(2024, 12, 31, 23, 59, 59)
    validate_date_range(start, end)


def test_validate_date_range_equal() -> None:
    dt = datetime(2024, 6, 15, 12, 0, 0)
    validate_date_range(dt, dt)


def test_validate_date_range_failure() -> None:
    start = datetime(2024, 12, 31, 23, 59, 59)
    end = datetime(2024, 1, 1, 0, 0, 0)
    with pytest.raises(ValidationError) as exc_info:
        validate_date_range(start, end)
    assert exc_info.value.code == "INVALID_DATE_RANGE"


# --- ValidationError code tests ---


def test_validation_error_code() -> None:
    with pytest.raises(ValidationError) as exc_info:
        validate_email("bad")
    assert exc_info.value.code == "INVALID_EMAIL"


def test_validation_error_default_code() -> None:
    err = ValidationError("custom_field", "some message")
    assert err.code == "INVALID_CUSTOM_FIELD"


# --- ValidationErrors collection tests ---


def test_validation_errors_empty() -> None:
    errors = ValidationErrors()
    assert not errors.has_errors()
    assert errors.get_errors() == []


def test_validation_errors_add_and_retrieve() -> None:
    errors = ValidationErrors()
    errors.add(ValidationError("email", "bad", code="INVALID_EMAIL"))
    errors.add(ValidationError("page", "bad", code="INVALID_PAGE"))

    assert errors.has_errors()
    assert len(errors.get_errors()) == 2
    assert errors.get_errors()[0].code == "INVALID_EMAIL"
    assert errors.get_errors()[1].code == "INVALID_PAGE"
