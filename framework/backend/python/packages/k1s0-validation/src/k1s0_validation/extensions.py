"""Validation helper functions for common k1s0 patterns."""

from __future__ import annotations

import re
import uuid

_KEBAB_CASE_PATTERN = re.compile(r"^[a-z][a-z0-9]*(-[a-z0-9]+)*$")


def validate_non_empty(value: str, field_name: str = "value") -> str:
    """Validate that a string is not empty or whitespace-only.

    Args:
        value: The string to validate.
        field_name: Name of the field for error messages.

    Returns:
        The stripped, non-empty string.

    Raises:
        ValueError: If the value is empty after stripping.
    """
    stripped = value.strip()
    if not stripped:
        msg = f"{field_name} must not be empty"
        raise ValueError(msg)
    return stripped


def validate_kebab_case(value: str, field_name: str = "value") -> str:
    """Validate that a string follows kebab-case convention.

    Args:
        value: The string to validate.
        field_name: Name of the field for error messages.

    Returns:
        The validated kebab-case string.

    Raises:
        ValueError: If the value is not valid kebab-case.
    """
    if not _KEBAB_CASE_PATTERN.match(value):
        msg = f"{field_name} must be kebab-case (e.g., 'my-service'), got: '{value}'"
        raise ValueError(msg)
    return value


def validate_uuid_str(value: str, field_name: str = "value") -> str:
    """Validate that a string is a valid UUID.

    Args:
        value: The string to validate.
        field_name: Name of the field for error messages.

    Returns:
        The validated UUID string (lowercase).

    Raises:
        ValueError: If the value is not a valid UUID.
    """
    try:
        parsed = uuid.UUID(value)
    except (ValueError, AttributeError):
        msg = f"{field_name} must be a valid UUID, got: '{value}'"
        raise ValueError(msg) from None
    return str(parsed)
