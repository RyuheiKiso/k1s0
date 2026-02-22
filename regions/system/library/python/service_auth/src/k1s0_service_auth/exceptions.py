"""service_auth ライブラリの例外型定義"""

from __future__ import annotations


class ServiceAuthError(Exception):
    """service_auth ライブラリのエラー基底クラス。"""

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


class ServiceAuthErrorCodes:
    """ServiceAuthError のエラーコード定数。"""

    TOKEN_REQUEST_FAILED: str = "TOKEN_REQUEST_FAILED"
    TOKEN_VALIDATION_FAILED: str = "TOKEN_VALIDATION_FAILED"
    UNAUTHORIZED: str = "UNAUTHORIZED"
