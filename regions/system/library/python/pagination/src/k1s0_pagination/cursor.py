"""Cursor-based pagination."""

from __future__ import annotations

import base64


def encode_cursor(id: str) -> str:
    """Encode an ID into a cursor string."""
    return base64.b64encode(id.encode()).decode()


def decode_cursor(cursor: str) -> str:
    """Decode a cursor string back to an ID."""
    return base64.b64decode(cursor.encode()).decode()
