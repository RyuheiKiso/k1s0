"""AES-GCM authenticated encryption.

Uses the ``cryptography`` library's AESGCM primitive with a random
12-byte nonce per encryption.  The wire format is::

    base64( nonce ‖ ciphertext ‖ tag )

where *nonce* is 12 bytes, and the 16-byte authentication tag is
appended by AESGCM automatically.
"""

from __future__ import annotations

import base64
import os

from cryptography.hazmat.primitives.ciphers.aead import AESGCM

_NONCE_SIZE = 12  # 96-bit nonce recommended by NIST for AES-GCM


def generate_key() -> bytes:
    """Generate a random 256-bit (32-byte) AES key."""
    return AESGCM.generate_key(bit_length=256)


def encrypt(key: bytes, plaintext: str) -> str:
    """Encrypt *plaintext* with AES-256-GCM and return a base64 string.

    A fresh random nonce is generated for every call so identical
    plaintexts produce different ciphertexts.

    Parameters
    ----------
    key:
        A 32-byte AES key (use :func:`generate_key`).
    plaintext:
        UTF-8 text to encrypt.

    Returns
    -------
    str
        Base64-encoded ``nonce + ciphertext + tag``.
    """
    nonce = os.urandom(_NONCE_SIZE)
    aesgcm = AESGCM(key)
    ct = aesgcm.encrypt(nonce, plaintext.encode("utf-8"), None)
    return base64.b64encode(nonce + ct).decode("ascii")


def decrypt(key: bytes, ciphertext: str) -> str:
    """Decrypt a base64 AES-GCM ciphertext produced by :func:`encrypt`.

    Parameters
    ----------
    key:
        The same 32-byte AES key used for encryption.
    ciphertext:
        Base64 string returned by :func:`encrypt`.

    Returns
    -------
    str
        The original plaintext.

    Raises
    ------
    cryptography.exceptions.InvalidTag
        If the key is wrong or the data has been tampered with.
    """
    raw = base64.b64decode(ciphertext)
    nonce = raw[:_NONCE_SIZE]
    ct = raw[_NONCE_SIZE:]
    aesgcm = AESGCM(key)
    return aesgcm.decrypt(nonce, ct, None).decode("utf-8")
