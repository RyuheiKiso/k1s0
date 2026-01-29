"""Error code validation following the k1s0 {service}.{category}.{reason} format."""

from __future__ import annotations

import re


class ErrorCode:
    """Represents a validated k1s0 error code.

    Error codes must follow the format: {service}.{category}.{reason}
    where each segment contains only lowercase letters, digits, and underscores.

    Examples:
        ErrorCode("auth.invalid_credentials")  -> "auth" service, "invalid_credentials" as category
        ErrorCode("user.profile.not_found")     -> "user" service, "profile" category, "not_found" reason
    """

    _PATTERN = re.compile(r"^[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*){1,2}$")

    def __init__(self, code: str) -> None:
        if not self._PATTERN.match(code):
            msg = (
                f"Invalid error code '{code}'. "
                "Must match format '{{service}}.{{category}}.{{reason}}' "
                "with lowercase letters, digits, and underscores."
            )
            raise ValueError(msg)
        self._code = code
        parts = code.split(".")
        self._service = parts[0]
        self._category = parts[1]
        self._reason = parts[2] if len(parts) > 2 else None

    @property
    def code(self) -> str:
        """Full error code string."""
        return self._code

    @property
    def service(self) -> str:
        """Service segment of the error code."""
        return self._service

    @property
    def category(self) -> str:
        """Category segment of the error code."""
        return self._category

    @property
    def reason(self) -> str | None:
        """Reason segment of the error code, if present."""
        return self._reason

    def __str__(self) -> str:
        return self._code

    def __repr__(self) -> str:
        return f"ErrorCode({self._code!r})"

    def __eq__(self, other: object) -> bool:
        if isinstance(other, ErrorCode):
            return self._code == other._code
        return NotImplemented

    def __hash__(self) -> int:
        return hash(self._code)
