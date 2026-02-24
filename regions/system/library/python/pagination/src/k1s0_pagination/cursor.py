"""Cursor-based pagination."""

from __future__ import annotations

import base64
from dataclasses import dataclass
from typing import Optional


@dataclass
class CursorRequest:
    """Cursor-based pagination request."""

    cursor: Optional[str]
    limit: int


@dataclass
class CursorMeta:
    """Cursor-based pagination response metadata."""

    next_cursor: Optional[str]
    has_more: bool


_CURSOR_SEPARATOR = "|"


def encode_cursor(sort_key: str, id: str) -> str:
    """Encode a sort key and id into a base64 cursor string."""
    combined = f"{sort_key}{_CURSOR_SEPARATOR}{id}"
    return base64.b64encode(combined.encode()).decode()


def decode_cursor(cursor: str) -> tuple[str, str]:
    """Decode a base64 cursor string into (sort_key, id)."""
    decoded = base64.b64decode(cursor.encode()).decode()
    idx = decoded.find(_CURSOR_SEPARATOR)
    if idx < 0:
        raise ValueError("invalid cursor: missing separator")
    return decoded[:idx], decoded[idx + 1 :]
