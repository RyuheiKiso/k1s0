"""Password hashing with Argon2id."""

from __future__ import annotations

from argon2 import PasswordHasher
from argon2.exceptions import InvalidHashError, VerifyMismatchError
from argon2.low_level import Type

_hasher = PasswordHasher(
    time_cost=2,
    memory_cost=19456,
    parallelism=1,
    hash_len=32,
    salt_len=16,
    type=Type.ID,
)


def hash_password(password: str) -> str:
    """Hash a password with Argon2id and random salt.

    Returns a PHC-format string:
    $argon2id$v=19$m=19456,t=2,p=1$<salt_base64>$<hash_base64>
    """
    return _hasher.hash(password)


def verify_password(password: str, hashed: str) -> bool:
    """Verify a password against an Argon2id hash."""
    try:
        return _hasher.verify(hashed, password)
    except (VerifyMismatchError, InvalidHashError):
        return False
