"""ServiceAuthClient 抽象基底クラス"""

from __future__ import annotations

from abc import ABC, abstractmethod

from .models import ServiceToken


class ServiceAuthClient(ABC):
    """サービス認証クライアント抽象基底クラス。"""

    @abstractmethod
    def get_token(self) -> ServiceToken:
        """アクセストークンを取得する（同期）。"""
        ...

    @abstractmethod
    async def get_token_async(self) -> ServiceToken:
        """アクセストークンを取得する（非同期）。"""
        ...

    @abstractmethod
    def get_cached_token(self) -> ServiceToken:
        """キャッシュされたトークンを取得する（期限切れの場合は自動更新）。"""
        ...

    @abstractmethod
    async def get_cached_token_async(self) -> ServiceToken:
        """非同期でキャッシュされたトークンを取得する（期限切れの場合は自動更新）。"""
        ...
