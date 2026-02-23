"""cache ライブラリの例外型定義"""

from __future__ import annotations


class CacheError(Exception):
    """cache ライブラリのエラー基底クラス。"""

    def __init__(self, code: str, message: str) -> None:
        super().__init__(message)
        self.code = code

    def __str__(self) -> str:
        return f"{self.code}: {super().__str__()}"


class CacheErrorCodes:
    """エラーコード定数。"""

    KEY_NOT_FOUND: str = "KEY_NOT_FOUND"
    CONNECTION_ERROR: str = "CONNECTION_ERROR"
    SERIALIZATION_ERROR: str = "SERIALIZATION_ERROR"
