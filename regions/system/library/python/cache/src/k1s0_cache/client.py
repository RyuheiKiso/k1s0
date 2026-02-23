"""CacheClient 抽象基底クラス"""

from __future__ import annotations

from abc import ABC, abstractmethod


class CacheClient(ABC):
    """キャッシュクライアント抽象基底クラス。"""

    @abstractmethod
    async def get(self, key: str) -> str | None:
        """キーに対応する値を取得する。存在しなければ None。"""
        ...

    @abstractmethod
    async def set(self, key: str, value: str, ttl: float | None = None) -> None:
        """キーと値を保存する。ttl 指定時は有効期限付き（秒）。"""
        ...

    @abstractmethod
    async def delete(self, key: str) -> bool:
        """キーを削除する。削除できたら True。"""
        ...

    @abstractmethod
    async def exists(self, key: str) -> bool:
        """キーが存在するか確認する。"""
        ...

    @abstractmethod
    async def set_nx(self, key: str, value: str, ttl: float) -> bool:
        """キーが存在しない場合のみ値を設定する。設定できたら True。"""
        ...
