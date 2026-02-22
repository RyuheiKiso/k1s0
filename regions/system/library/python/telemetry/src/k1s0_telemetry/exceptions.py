"""telemetry ライブラリの例外型定義"""

from __future__ import annotations


class TelemetryError(Exception):
    """telemetry ライブラリのエラー基底クラス。"""

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


class TelemetryErrorCodes:
    """TelemetryError のエラーコード定数。"""

    INITIALIZATION_ERROR: str = "INITIALIZATION_ERROR"
    EXPORT_ERROR: str = "EXPORT_ERROR"
