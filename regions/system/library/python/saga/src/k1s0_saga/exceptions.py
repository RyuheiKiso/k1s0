"""saga ライブラリの例外型定義"""

from __future__ import annotations


class SagaError(Exception):
    """saga ライブラリのエラー基底クラス。"""

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


class SagaErrorCodes:
    """SagaError のエラーコード定数。"""

    SAGA_NOT_FOUND: str = "SAGA_NOT_FOUND"
    HTTP_ERROR: str = "HTTP_ERROR"
    INVALID_STATE: str = "INVALID_STATE"
    CANCEL_FAILED: str = "CANCEL_FAILED"
