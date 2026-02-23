"""File storage client for k1s0 platform."""

from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from datetime import datetime, timedelta, timezone


class FileClientError(Exception):
    """File client error."""

    def __init__(self, message: str, code: str) -> None:
        super().__init__(message)
        self.code = code


@dataclass
class FileMetadata:
    """File metadata."""

    path: str
    size_bytes: int
    content_type: str
    etag: str
    last_modified: datetime
    tags: dict[str, str] = field(default_factory=dict)


@dataclass
class PresignedUrl:
    """Presigned URL for file upload/download."""

    url: str
    method: str
    expires_at: datetime
    headers: dict[str, str] = field(default_factory=dict)


class FileClient(ABC):
    """Abstract file client."""

    @abstractmethod
    async def generate_upload_url(
        self, path: str, content_type: str, expires_in: timedelta
    ) -> PresignedUrl: ...

    @abstractmethod
    async def generate_download_url(
        self, path: str, expires_in: timedelta
    ) -> PresignedUrl: ...

    @abstractmethod
    async def delete(self, path: str) -> None: ...

    @abstractmethod
    async def get_metadata(self, path: str) -> FileMetadata: ...

    @abstractmethod
    async def list(self, prefix: str) -> list[FileMetadata]: ...

    @abstractmethod
    async def copy(self, src: str, dst: str) -> None: ...


class InMemoryFileClient(FileClient):
    """In-memory file client for testing."""

    def __init__(self) -> None:
        self._files: dict[str, FileMetadata] = {}

    @property
    def stored_files(self) -> list[FileMetadata]:
        """Get a copy of stored files."""
        return list(self._files.values())

    async def generate_upload_url(
        self, path: str, content_type: str, expires_in: timedelta
    ) -> PresignedUrl:
        now = datetime.now(timezone.utc)
        self._files[path] = FileMetadata(
            path=path,
            size_bytes=0,
            content_type=content_type,
            etag="",
            last_modified=now,
        )
        return PresignedUrl(
            url=f"https://storage.example.com/upload/{path}",
            method="PUT",
            expires_at=now + expires_in,
        )

    async def generate_download_url(
        self, path: str, expires_in: timedelta
    ) -> PresignedUrl:
        if path not in self._files:
            raise FileClientError(f"File not found: {path}", "NOT_FOUND")
        now = datetime.now(timezone.utc)
        return PresignedUrl(
            url=f"https://storage.example.com/download/{path}",
            method="GET",
            expires_at=now + expires_in,
        )

    async def delete(self, path: str) -> None:
        if path not in self._files:
            raise FileClientError(f"File not found: {path}", "NOT_FOUND")
        del self._files[path]

    async def get_metadata(self, path: str) -> FileMetadata:
        if path not in self._files:
            raise FileClientError(f"File not found: {path}", "NOT_FOUND")
        return self._files[path]

    async def list(self, prefix: str) -> list[FileMetadata]:
        return [f for f in self._files.values() if f.path.startswith(prefix)]

    async def copy(self, src: str, dst: str) -> None:
        if src not in self._files:
            raise FileClientError(f"File not found: {src}", "NOT_FOUND")
        source = self._files[src]
        self._files[dst] = FileMetadata(
            path=dst,
            size_bytes=source.size_bytes,
            content_type=source.content_type,
            etag=source.etag,
            last_modified=source.last_modified,
            tags=dict(source.tags),
        )
