"""Configuration model for JWT verification."""

from __future__ import annotations

from pydantic import BaseModel


class JwtVerifierConfig(BaseModel):
    """Configuration for :class:`k1s0_auth.jwt.verifier.JwtVerifier`.

    Attributes:
        issuer: Expected ``iss`` claim value.
        jwks_uri: URL to fetch the JSON Web Key Set from.
        audience: Expected ``aud`` claim value.
        clock_skew: Allowed clock skew in seconds for expiry validation.
        algorithms: Accepted signing algorithms.
    """

    issuer: str
    jwks_uri: str
    audience: str
    clock_skew: int = 30
    algorithms: list[str] = ["RS256"]
