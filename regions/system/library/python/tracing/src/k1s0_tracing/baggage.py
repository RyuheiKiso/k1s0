"""W3C Baggage propagation."""

from __future__ import annotations


class Baggage:
    """W3C Baggage container."""

    def __init__(self) -> None:
        self._entries: dict[str, str] = {}

    def set(self, key: str, value: str) -> None:
        self._entries[key] = value

    def get(self, key: str) -> str | None:
        return self._entries.get(key)

    def to_header(self) -> str:
        return ",".join(f"{k}={v}" for k, v in self._entries.items())

    @classmethod
    def from_header(cls, s: str) -> Baggage:
        b = cls()
        if not s:
            return b
        for item in s.split(","):
            if "=" in item:
                k, _, v = item.partition("=")
                b.set(k.strip(), v.strip())
        return b

    @property
    def is_empty(self) -> bool:
        return len(self._entries) == 0
