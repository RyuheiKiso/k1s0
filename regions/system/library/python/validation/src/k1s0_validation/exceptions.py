"""Validation exceptions."""

from __future__ import annotations


class ValidationError(Exception):
    """Validation error with field name, message, and code."""

    def __init__(self, field: str, message: str, *, code: str | None = None) -> None:
        self.field = field
        self.message = message
        self.code = code if code is not None else f"INVALID_{field.upper()}"
        super().__init__(f"ValidationError({field}, {self.code}): {message}")


class ValidationErrors:
    """A collection of ValidationError instances."""

    def __init__(self) -> None:
        self._errors: list[ValidationError] = []

    def has_errors(self) -> bool:
        """Returns True if there are any errors."""
        return len(self._errors) > 0

    def get_errors(self) -> list[ValidationError]:
        """Returns a copy of all collected errors."""
        return list(self._errors)

    def add(self, error: ValidationError) -> None:
        """Adds a validation error to the collection."""
        self._errors.append(error)
