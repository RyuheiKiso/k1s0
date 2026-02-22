"""kafka ライブラリの例外型定義"""

from __future__ import annotations


class KafkaError(Exception):
    """kafka ライブラリのエラー基底クラス。"""

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


class KafkaErrorCodes:
    """KafkaError のエラーコード定数。"""

    CONNECTION: str = "CONNECTION_ERROR"
    TOPIC_NOT_FOUND: str = "TOPIC_NOT_FOUND"
    PARTITION: str = "PARTITION_ERROR"
    CONFIG: str = "CONFIG_ERROR"
    TIMEOUT: str = "TIMEOUT_ERROR"
