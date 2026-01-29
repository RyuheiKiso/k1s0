"""Configuration wrapper with typed accessors."""

from __future__ import annotations

from typing import TypeVar

T = TypeVar("T")


class K1s0Config:
    """Wrapper around a configuration dictionary providing typed access.

    Supports dot-notation key paths for nested values (e.g., "database.host").
    """

    def __init__(self, data: dict[str, object]) -> None:
        self._data = data

    def get(self, key: str, default: T = None) -> object | T:  # type: ignore[assignment]
        """Get a configuration value by dot-separated key path.

        Args:
            key: Dot-separated key path (e.g., "database.host").
            default: Value to return if key is not found.

        Returns:
            The configuration value or the default.
        """
        parts = key.split(".")
        current: object = self._data
        for part in parts:
            if not isinstance(current, dict):
                return default
            current = current.get(part)  # type: ignore[union-attr]
            if current is None:
                return default
        return current

    def get_str(self, key: str, default: str = "") -> str:
        """Get a string configuration value."""
        value = self.get(key, default)
        return str(value)

    def get_int(self, key: str, default: int = 0) -> int:
        """Get an integer configuration value."""
        value = self.get(key, default)
        return int(value)  # type: ignore[arg-type]

    def get_bool(self, key: str, default: bool = False) -> bool:  # noqa: FBT001, FBT002
        """Get a boolean configuration value."""
        value = self.get(key, default)
        return bool(value)

    def get_section(self, key: str) -> K1s0Config:
        """Get a nested section as a new K1s0Config instance."""
        value = self.get(key)
        if isinstance(value, dict):
            return K1s0Config(value)  # type: ignore[arg-type]
        return K1s0Config({})

    @property
    def raw(self) -> dict[str, object]:
        """Access the underlying raw dictionary."""
        return self._data
