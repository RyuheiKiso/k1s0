"""auth ライブラリの例外型定義"""

from __future__ import annotations


class AuthError(Exception):
    """auth ライブラリのエラー基底クラス。"""

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


class AuthErrorCodes:
    """AuthError のエラーコード定数。"""

    INVALID_TOKEN: str = "INVALID_TOKEN"
    EXPIRED_TOKEN: str = "EXPIRED_TOKEN"
    UNAUTHORIZED: str = "UNAUTHORIZED"
    JWKS_FETCH_ERROR: str = "JWKS_FETCH_ERROR"
    DEVICE_FLOW_ERROR: str = "DEVICE_FLOW_ERROR"
    PKCE_ERROR: str = "PKCE_ERROR"
