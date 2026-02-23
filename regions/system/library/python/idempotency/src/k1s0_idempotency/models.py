"""idempotency データモデル"""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime, timezone
from enum import Enum


class IdempotencyStatus(Enum):
    """冪等リクエストの状態。"""

    PENDING = "pending"
    COMPLETED = "completed"
    FAILED = "failed"


@dataclass
class IdempotencyRecord:
    """冪等レコード。"""

    key: str
    status: IdempotencyStatus = IdempotencyStatus.PENDING
    response_body: str | None = None
    status_code: int | None = None
    created_at: datetime = field(default_factory=lambda: datetime.now(timezone.utc))
    expires_at: datetime | None = None
    completed_at: datetime | None = None

    def is_expired(self) -> bool:
        """有効期限が切れているか確認する。"""
        if self.expires_at is None:
            return False
        return self.expires_at <= datetime.now(timezone.utc)
