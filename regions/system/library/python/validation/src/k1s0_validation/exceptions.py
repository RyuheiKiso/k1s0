"""Validation exceptions."""


class ValidationError(Exception):
    """Validation error with field name and message."""

    def __init__(self, field: str, message: str) -> None:
        self.field = field
        self.message = message
        super().__init__(f"ValidationError({field}): {message}")
