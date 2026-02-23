"""InMemoryIdempotencyStore 実装"""

from __future__ import annotations

from datetime import datetime, timezone

from .exceptions import DuplicateKeyError
from .models import IdempotencyRecord, IdempotencyStatus
from .store import IdempotencyStore


class InMemoryIdempotencyStore(IdempotencyStore):
    """テスト用インメモリ冪等ストア。"""

    def __init__(self) -> None:
        self._records: dict[str, IdempotencyRecord] = {}

    async def get(self, key: str) -> IdempotencyRecord | None:
        record = self._records.get(key)
        if record is not None and record.is_expired():
            del self._records[key]
            return None
        return record

    async def insert(self, record: IdempotencyRecord) -> None:
        existing = await self.get(record.key)
        if existing is not None:
            raise DuplicateKeyError(record.key)
        self._records[record.key] = record

    async def update(
        self,
        key: str,
        status: IdempotencyStatus,
        body: str | None = None,
        code: int | None = None,
    ) -> None:
        record = self._records.get(key)
        if record is None:
            return
        record.status = status
        if body is not None:
            record.response_body = body
        if code is not None:
            record.status_code = code
        if status in (IdempotencyStatus.COMPLETED, IdempotencyStatus.FAILED):
            record.completed_at = datetime.now(timezone.utc)

    async def delete(self, key: str) -> bool:
        if key in self._records:
            del self._records[key]
            return True
        return False
