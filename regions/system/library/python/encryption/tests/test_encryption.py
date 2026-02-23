"""encryption library unit tests."""

from k1s0_encryption import (
    decrypt,
    encrypt,
    generate_key,
    hash_password,
    verify_password,
)


def test_hash_password_format() -> None:
    hashed = hash_password("secret")
    assert ":" in hashed
    salt, h = hashed.split(":", 1)
    assert len(salt) == 32  # 16 bytes hex
    assert len(h) == 64  # SHA256 hex


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


def test_generate_key_length() -> None:
    key = generate_key()
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
