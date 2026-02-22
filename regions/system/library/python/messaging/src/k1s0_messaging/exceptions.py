"""messaging ライブラリの例外型定義"""

from __future__ import annotations


class MessagingError(Exception):
    """messaging ライブラリのエラー基底クラス。"""

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


class MessagingErrorCodes:
    """MessagingError のエラーコード定数。"""

    PUBLISH_FAILED: str = "PUBLISH_FAILED"
    RECEIVE_FAILED: str = "RECEIVE_FAILED"
    CONNECTION_FAILED: str = "CONNECTION_FAILED"
    SERIALIZATION_ERROR: str = "SERIALIZATION_ERROR"
