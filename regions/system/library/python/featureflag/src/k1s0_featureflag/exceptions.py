"""featureflag ライブラリの例外型定義"""

from __future__ import annotations


class FeatureFlagError(Exception):
    """featureflag ライブラリのエラー基底クラス。"""

    def __init__(self, code: str, message: str) -> None:
        super().__init__(message)
        self.code = code

    def __str__(self) -> str:
        return f"{self.code}: {super().__str__()}"


class FeatureFlagErrorCodes:
    """エラーコード定数。"""

    FLAG_NOT_FOUND: str = "FLAG_NOT_FOUND"
    CONNECTION_ERROR: str = "CONNECTION_ERROR"
    CONFIG_ERROR: str = "CONFIG_ERROR"
