"""Domain event error hierarchy for k1s0."""

from __future__ import annotations


class DomainEventError(Exception):
    """Base exception for all domain event errors."""

    def __init__(self, message: str) -> None:
        self.message = message
        super().__init__(message)


class PublishError(DomainEventError):
    """Raised when publishing a domain event fails."""


class SubscribeError(DomainEventError):
    """Raised when subscribing to domain events fails."""


class OutboxError(DomainEventError):
    """Raised when an outbox operation fails."""
