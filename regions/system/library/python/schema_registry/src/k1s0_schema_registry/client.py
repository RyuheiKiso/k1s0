"""Schema Registry クライアント抽象基底クラス"""

from __future__ import annotations

from abc import ABC, abstractmethod

from .models import RegisteredSchema, SchemaType


class SchemaRegistryClient(ABC):
    """Schema Registry クライアント抽象基底クラス。"""

    @abstractmethod
    def register_schema(
        self,
        subject: str,
        schema: str,
        schema_type: SchemaType = SchemaType.AVRO,
    ) -> int:
        """スキーマを登録してスキーマ ID を返す。"""
        ...

    @abstractmethod
    async def register_schema_async(
        self,
        subject: str,
        schema: str,
        schema_type: SchemaType = SchemaType.AVRO,
    ) -> int:
        """非同期でスキーマを登録してスキーマ ID を返す。"""
        ...

    @abstractmethod
    def get_schema_by_id(self, schema_id: int) -> RegisteredSchema:
        """ID でスキーマを取得する。"""
        ...

    @abstractmethod
    async def get_schema_by_id_async(self, schema_id: int) -> RegisteredSchema:
        """非同期で ID でスキーマを取得する。"""
        ...

    @abstractmethod
    def check_compatibility(self, subject: str, schema: str) -> bool:
        """スキーマの互換性チェックを実行する。"""
        ...

    @abstractmethod
    async def check_compatibility_async(self, subject: str, schema: str) -> bool:
        """非同期でスキーマの互換性チェックを実行する。"""
        ...
