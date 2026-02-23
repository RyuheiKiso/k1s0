"""Quota client configuration."""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import timedelta


@dataclass
class QuotaClientConfig:
    """Configuration for quota client."""

    server_url: str
    timeout: timedelta = field(default_factory=lambda: timedelta(seconds=5))
    policy_cache_ttl: timedelta = field(default_factory=lambda: timedelta(seconds=60))
