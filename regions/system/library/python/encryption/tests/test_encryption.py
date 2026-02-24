"""encryption library unit tests."""

from __future__ import annotations

import base64

import pytest
from cryptography.exceptions import InvalidTag

from k1s0_encryption import (
    decrypt,
    encrypt,
    generate_key,
    hash_password,
    verify_password,
)


# ---- password hashing ---------------------------------------------------


def test_hash_password_format() -> None:
    hashed = hash_password("secret")
    assert hashed.startswith("$argon2id$")
    assert "m=19456,t=2,p=1" in hashed


def test_verify_password_success() -> None:
    hashed = hash_password("secret")
    assert verify_password("secret", hashed) is True


def test_verify_password_failure() -> None:
    hashed = hash_password("secret")
    assert verify_password("wrong", hashed) is False


def test_hash_password_unique_salts() -> None:
    h1 = hash_password("same")
    h2 = hash_password("same")
    assert h1 != h2


# ---- AES-GCM encryption -------------------------------------------------


def test_generate_key_length() -> None:
    key = generate_key()
    assert isinstance(key, bytes)
    assert len(key) == 32


def test_encrypt_decrypt_roundtrip() -> None:
    key = generate_key()
    plaintext = "hello, world!"
    ciphertext = encrypt(key, plaintext)
    assert decrypt(key, ciphertext) == plaintext


def test_encrypt_returns_different_from_plaintext() -> None:
    key = generate_key()
    plaintext = "hello, world!"
    ciphertext = encrypt(key, plaintext)
    assert ciphertext != plaintext


def test_encrypt_produces_unique_ciphertexts() -> None:
    """Each call must use a fresh nonce, so two encryptions of the same
    plaintext with the same key must produce different ciphertexts."""
    key = generate_key()
    ct1 = encrypt(key, "same")
    ct2 = encrypt(key, "same")
    assert ct1 != ct2


def test_ciphertext_is_valid_base64() -> None:
    key = generate_key()
    ct = encrypt(key, "test")
    raw = base64.b64decode(ct)
    # nonce (12) + ciphertext (len("test")=4) + tag (16) = 32 bytes
    assert len(raw) == 12 + 4 + 16


def test_decrypt_with_wrong_key_raises() -> None:
    key1 = generate_key()
    key2 = generate_key()
    ct = encrypt(key1, "secret")
    with pytest.raises(InvalidTag):
        decrypt(key2, ct)


def test_tampered_ciphertext_raises() -> None:
    key = generate_key()
    ct = encrypt(key, "important data")
    raw = bytearray(base64.b64decode(ct))
    # flip a byte in the ciphertext portion (after the 12-byte nonce)
    raw[12] ^= 0xFF
    tampered = base64.b64encode(bytes(raw)).decode("ascii")
    with pytest.raises(InvalidTag):
        decrypt(key, tampered)


def test_roundtrip_empty_string() -> None:
    key = generate_key()
    assert decrypt(key, encrypt(key, "")) == ""


def test_roundtrip_unicode() -> None:
    key = generate_key()
    plaintext = "こんにちは世界 🌍"
    assert decrypt(key, encrypt(key, plaintext)) == plaintext


def test_roundtrip_long_text() -> None:
    key = generate_key()
    plaintext = "A" * 10_000
    assert decrypt(key, encrypt(key, plaintext)) == plaintext
