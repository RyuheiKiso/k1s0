"""retry ライブラリの例外型定義"""

from __future__ import annotations


class RetryError(Exception):
    """リトライ上限に達した場合のエラー。"""

    def __init__(self, attempts: int, last_error: Exception | None = None) -> None:
        self.attempts = attempts
        self.last_error = last_error
        msg = f"リトライ上限 ({attempts} 回) に達しました"
        if last_error:
            msg += f": {last_error}"
        super().__init__(msg)
        if last_error is not None:
            self.__cause__ = last_error
