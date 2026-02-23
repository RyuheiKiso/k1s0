"""Password hashing with PBKDF2-SHA256."""

from __future__ import annotations

import hashlib
import secrets


def hash_password(password: str) -> str:
    """Hash a password with PBKDF2-SHA256 and random salt."""
    salt = secrets.token_hex(16)
    h = hashlib.pbkdf2_hmac("sha256", password.encode(), salt.encode(), 100_000)
    return f"{salt}:{h.hex()}"


def verify_password(password: str, hashed: str) -> bool:
    """Verify a password against a PBKDF2-SHA256 hash."""
    salt, stored = hashed.split(":", 1)
    h = hashlib.pbkdf2_hmac("sha256", password.encode(), salt.encode(), 100_000)
    return secrets.compare_digest(h.hex(), stored)
