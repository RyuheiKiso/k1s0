"""config ライブラリの例外型定義"""

from __future__ import annotations


class ConfigError(Exception):
    """config ライブラリのエラー基底クラス。"""

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


class ConfigErrorCodes:
    """ConfigError のエラーコード定数。"""

    READ_FILE: str = "READ_FILE_ERROR"
    PARSE_YAML: str = "PARSE_YAML_ERROR"
    VALIDATION: str = "VALIDATION_ERROR"
