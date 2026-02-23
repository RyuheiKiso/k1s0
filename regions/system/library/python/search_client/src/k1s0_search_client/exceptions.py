"""Search client exceptions."""

from __future__ import annotations

from enum import Enum


class SearchErrorCode(str, Enum):
    """Search error codes."""

    INDEX_NOT_FOUND = "index_not_found"
    INVALID_QUERY = "invalid_query"
    SERVER_ERROR = "server_error"
    TIMEOUT = "timeout"


class SearchError(Exception):
    """Search client error."""

    def __init__(self, message: str, code: SearchErrorCode) -> None:
        super().__init__(message)
        self.code = code
