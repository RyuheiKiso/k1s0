"""Quota data models."""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
from enum import Enum


class QuotaPeriod(str, Enum):
    """Quota period types."""

    HOURLY = "hourly"
    DAILY = "daily"
    MONTHLY = "monthly"
    CUSTOM = "custom"


@dataclass
class QuotaStatus:
    """Quota check result."""

    allowed: bool
    remaining: int
    limit: int
    reset_at: datetime


@dataclass
class QuotaUsage:
    """Quota usage."""

    quota_id: str
    used: int
    limit: int
    period: QuotaPeriod
    reset_at: datetime


@dataclass
class QuotaPolicy:
    """Quota policy."""

    quota_id: str
    limit: int
    period: QuotaPeriod
    reset_strategy: str
