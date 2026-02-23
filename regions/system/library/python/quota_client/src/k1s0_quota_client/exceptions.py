"""Quota client exceptions."""

from __future__ import annotations


class QuotaClientError(Exception):
    """Base quota client error."""


class QuotaExceededError(QuotaClientError):
    """Quota exceeded error."""

    def __init__(self, quota_id: str, remaining: int) -> None:
        self.quota_id = quota_id
        self.remaining = remaining
        super().__init__(f"Quota exceeded: {quota_id}, remaining={remaining}")


class QuotaNotFoundError(QuotaClientError):
    """Quota not found error."""

    def __init__(self, quota_id: str) -> None:
        self.quota_id = quota_id
        super().__init__(f"Quota not found: {quota_id}")
