"""Structured audit logging for authentication and authorization events."""

from __future__ import annotations

import logging


class AuditLogger:
    """Logs authentication and authorization events.

    Args:
        logger_name: The name for the underlying Python logger.
    """

    def __init__(self, logger_name: str = "k1s0.auth.audit") -> None:
        self._logger = logging.getLogger(logger_name)

    def log_authentication(
        self,
        sub: str,
        success: bool,
        reason: str | None = None,
    ) -> None:
        """Log an authentication attempt.

        Args:
            sub: Subject identifier.
            success: Whether authentication succeeded.
            reason: Optional failure reason.
        """
        extra = {"sub": sub, "success": success, "event": "authentication"}
        if reason:
            extra["reason"] = reason  # type: ignore[assignment]
        if success:
            self._logger.info("Authentication succeeded for %s", sub, extra=extra)
        else:
            self._logger.warning("Authentication failed for %s: %s", sub, reason, extra=extra)

    def log_authorization(
        self,
        sub: str,
        action: str,
        resource: str,
        allowed: bool,
    ) -> None:
        """Log an authorization decision.

        Args:
            sub: Subject identifier.
            action: The requested action.
            resource: The target resource.
            allowed: Whether access was granted.
        """
        extra = {
            "sub": sub,
            "action": action,
            "resource": resource,
            "allowed": allowed,
            "event": "authorization",
        }
        level = logging.INFO if allowed else logging.WARNING
        self._logger.log(
            level,
            "Authorization %s for %s: %s on %s",
            "granted" if allowed else "denied",
            sub,
            action,
            resource,
            extra=extra,
        )
