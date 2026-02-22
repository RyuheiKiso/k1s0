"""dlq_client ライブラリの例外型定義"""

from __future__ import annotations


class DlqClientError(Exception):
    """dlq_client ライブラリのエラー基底クラス。"""

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


class DlqClientErrorCodes:
    """DlqClientError のエラーコード定数。"""

    MESSAGE_NOT_FOUND: str = "MESSAGE_NOT_FOUND"
    HTTP_ERROR: str = "HTTP_ERROR"
    UNAUTHORIZED: str = "UNAUTHORIZED"
