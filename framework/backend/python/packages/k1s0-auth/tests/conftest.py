"""Shared test fixtures for k1s0-auth tests."""

from __future__ import annotations

import json
import time
from typing import Any

import pytest
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric import rsa

import jwt as pyjwt
from jwt import PyJWK


@pytest.fixture()
def rsa_keypair() -> tuple[rsa.RSAPrivateKey, rsa.RSAPublicKey]:
    """Generate a test RSA key pair."""
    private_key = rsa.generate_private_key(public_exponent=65537, key_size=2048)
    public_key = private_key.public_key()
    return private_key, public_key


@pytest.fixture()
def private_key_pem(rsa_keypair: tuple[rsa.RSAPrivateKey, rsa.RSAPublicKey]) -> bytes:
    """PEM-encoded private key."""
    return rsa_keypair[0].private_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PrivateFormat.PKCS8,
        encryption_algorithm=serialization.NoEncryption(),
    )


@pytest.fixture()
def jwks_response(rsa_keypair: tuple[rsa.RSAPrivateKey, rsa.RSAPublicKey]) -> dict[str, Any]:
    """JWKS JSON response containing the test public key."""
    public_key = rsa_keypair[1]
    # Use PyJWT to create the JWK representation
    from jwt.algorithms import RSAAlgorithm

    jwk_dict = json.loads(RSAAlgorithm.to_jwk(public_key))
    jwk_dict["kid"] = "test-key-1"
    jwk_dict["use"] = "sig"
    jwk_dict["alg"] = "RS256"
    return {"keys": [jwk_dict]}


@pytest.fixture()
def make_token(private_key_pem: bytes) -> Any:
    """Factory fixture to create signed JWT tokens."""

    def _make(
        sub: str = "user-123",
        roles: list[str] | None = None,
        permissions: list[str] | None = None,
        issuer: str = "https://auth.example.com",
        audience: str = "my-api",
        exp_offset: int = 3600,
        extra: dict[str, Any] | None = None,
    ) -> str:
        now = time.time()
        payload: dict[str, Any] = {
            "sub": sub,
            "iss": issuer,
            "aud": audience,
            "iat": now,
            "exp": now + exp_offset,
            "roles": roles or [],
            "permissions": permissions or [],
        }
        if extra:
            payload.update(extra)
        return pyjwt.encode(
            payload,
            private_key_pem,
            algorithm="RS256",
            headers={"kid": "test-key-1"},
        )

    return _make
