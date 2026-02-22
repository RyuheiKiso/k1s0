"""outbox ライブラリの例外型定義"""

from __future__ import annotations


class OutboxError(Exception):
    """outbox ライブラリのエラー基底クラス。"""

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


class OutboxErrorCodes:
    """OutboxError のエラーコード定数。"""

    SAVE_FAILED: str = "SAVE_FAILED"
    FETCH_FAILED: str = "FETCH_FAILED"
    UPDATE_FAILED: str = "UPDATE_FAILED"
    PUBLISH_FAILED: str = "PUBLISH_FAILED"
