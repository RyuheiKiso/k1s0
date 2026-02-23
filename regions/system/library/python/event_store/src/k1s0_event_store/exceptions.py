"""event_store ライブラリの例外型定義"""

from __future__ import annotations


class VersionConflictError(Exception):
    """楽観的排他制御の競合エラー。"""

    def __init__(self, expected: int, actual: int) -> None:
        self.expected = expected
        self.actual = actual
        super().__init__(f"version conflict: expected={expected}, actual={actual}")


class StreamNotFoundError(Exception):
    """ストリームが見つからない場合のエラー。"""

    def __init__(self, stream_id: str) -> None:
        self.stream_id = stream_id
        super().__init__(f"stream not found: {stream_id}")
