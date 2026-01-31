"""Domain-specific exception hierarchy for authentication and authorization."""

from __future__ import annotations


class AuthError(Exception):
    """Base exception for all authentication and authorization errors.

    Attributes:
        error_code: A structured error code (e.g. ``auth.token_expired``).
    """

    def __init__(self, message: str, error_code: str = "auth.unknown") -> None:
        super().__init__(message)
        self.error_code = error_code


class TokenExpiredError(AuthError):
    """Raised when a JWT token has expired."""

    def __init__(self, message: str = "Token has expired") -> None:
        super().__init__(message, error_code="auth.token_expired")


class TokenInvalidError(AuthError):
    """Raised when a JWT token is malformed or has an invalid signature."""

    def __init__(self, message: str = "Token is invalid") -> None:
        super().__init__(message, error_code="auth.token_invalid")


class InsufficientPermissionError(AuthError):
    """Raised when the authenticated subject lacks required permissions."""

    def __init__(self, message: str = "Insufficient permissions") -> None:
        super().__init__(message, error_code="auth.insufficient_permission")


class DiscoveryError(AuthError):
    """Raised when OIDC discovery fails."""

    def __init__(self, message: str = "OIDC discovery failed") -> None:
        super().__init__(message, error_code="auth.discovery_failed")
