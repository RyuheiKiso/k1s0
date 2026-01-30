"""k1s0-auth: Authentication and authorization for k1s0 Python services."""

from __future__ import annotations

from k1s0_auth.audit import AuditLogger
from k1s0_auth.blacklist import InMemoryBlacklist, TokenBlacklist
from k1s0_auth.errors import (
    AuthError,
    DiscoveryError,
    InsufficientPermissionError,
    TokenExpiredError,
    TokenInvalidError,
)
from k1s0_auth.jwt.claims import Claims
from k1s0_auth.jwt.config import JwtVerifierConfig
from k1s0_auth.jwt.verifier import JwtVerifier
from k1s0_auth.oidc.discovery import OidcDiscovery
from k1s0_auth.oidc.verifier import OidcJwtVerifier
from k1s0_auth.policy.evaluator import PolicyEvaluator, RepositoryPolicyEvaluator
from k1s0_auth.policy.models import Action, PolicyRequest, PolicyRule, PolicySubject
from k1s0_auth.policy.repository import InMemoryPolicyRepository, PolicyRepository
from k1s0_auth.refresh import RefreshTokenManager, RefreshTokenStore

__all__ = [
    "Action",
    "AuditLogger",
    "AuthError",
    "Claims",
    "DiscoveryError",
    "InMemoryBlacklist",
    "InMemoryPolicyRepository",
    "InsufficientPermissionError",
    "JwtVerifier",
    "JwtVerifierConfig",
    "OidcDiscovery",
    "OidcJwtVerifier",
    "PolicyEvaluator",
    "PolicyRepository",
    "PolicyRequest",
    "PolicyRule",
    "PolicySubject",
    "RefreshTokenManager",
    "RefreshTokenStore",
    "RepositoryPolicyEvaluator",
    "TokenBlacklist",
    "TokenExpiredError",
    "TokenInvalidError",
]
