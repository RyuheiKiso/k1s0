"""idempotency ライブラリの例外型定義"""

from __future__ import annotations


class DuplicateKeyError(Exception):
    """キーが既に存在する場合のエラー。"""

    def __init__(self, key: str) -> None:
        self.key = key
        super().__init__(f"キーが既に存在します: {key}")
