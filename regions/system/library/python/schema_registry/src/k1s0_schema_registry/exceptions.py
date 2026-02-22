"""schema_registry ライブラリの例外型定義"""

from __future__ import annotations


class SchemaRegistryError(Exception):
    """schema_registry ライブラリのエラー基底クラス。"""

    def __init__(
        self,
        code: str,
        message: str,
        cause: Exception | None = None,
    ) -> None:
        super().__init__(message)
        self.code = code
        if cause is not None:
            self.__cause__ = cause

    def __str__(self) -> str:
        return f"{self.code}: {super().__str__()}"


class SchemaRegistryErrorCodes:
    """SchemaRegistryError のエラーコード定数。"""

    SCHEMA_NOT_FOUND: str = "SCHEMA_NOT_FOUND"
    HTTP_ERROR: str = "HTTP_ERROR"
    COMPATIBILITY_ERROR: str = "COMPATIBILITY_ERROR"
    SERIALIZATION_ERROR: str = "SERIALIZATION_ERROR"
