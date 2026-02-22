"""correlation ライブラリの例外型定義"""

from __future__ import annotations


class CorrelationError(Exception):
    """correlation ライブラリのエラー基底クラス。"""

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


class CorrelationErrorCodes:
    """CorrelationError のエラーコード定数。"""

    INVALID_HEADER: str = "INVALID_HEADER"
    CONTEXT_ERROR: str = "CONTEXT_ERROR"
