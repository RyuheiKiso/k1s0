"""AES-like encryption (mock implementation using base64).

Note: Python 3.12 stdlib does not include AES-GCM.
For production use, the `cryptography` library is recommended.
"""

from __future__ import annotations

import base64
import os


def generate_key() -> bytes:
    """Generate a random 32-byte key."""
    return os.urandom(32)


def encrypt(key: bytes, plaintext: str) -> str:
    """Encrypt plaintext (mock: base64 encode)."""
    return base64.b64encode(plaintext.encode()).decode()


def decrypt(key: bytes, ciphertext: str) -> str:
    """Decrypt ciphertext (mock: base64 decode)."""
    return base64.b64decode(ciphertext.encode()).decode()
