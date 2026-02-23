"""IdempotencyStore 抽象基底クラス"""

from __future__ import annotations

from abc import ABC, abstractmethod

from .models import IdempotencyRecord, IdempotencyStatus


class IdempotencyStore(ABC):
    """冪等ストア抽象基底クラス。"""

    @abstractmethod
    async def get(self, key: str) -> IdempotencyRecord | None:
        """キーに対応するレコードを取得する。"""
        ...

    @abstractmethod
    async def insert(self, record: IdempotencyRecord) -> None:
        """レコードを挿入する。キーが既に存在する場合は DuplicateKeyError。"""
        ...

    @abstractmethod
    async def update(
        self,
        key: str,
        status: IdempotencyStatus,
        body: str | None = None,
        code: int | None = None,
    ) -> None:
        """レコードのステータスを更新する。"""
        ...

    @abstractmethod
    async def delete(self, key: str) -> bool:
        """レコードを削除する。"""
        ...
