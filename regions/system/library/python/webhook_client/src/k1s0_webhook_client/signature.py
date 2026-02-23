"""Webhook signature generation and verification."""

from __future__ import annotations

import hashlib
import hmac


def generate_signature(secret: str, body: bytes) -> str:
    """Generate HMAC-SHA256 signature for a webhook body."""
    return hmac.new(secret.encode(), body, hashlib.sha256).hexdigest()


def verify_signature(secret: str, body: bytes, signature: str) -> bool:
    """Verify a webhook signature."""
    expected = generate_signature(secret, body)
    return hmac.compare_digest(expected, signature)
