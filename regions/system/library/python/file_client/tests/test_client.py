"""file_client library unit tests."""

import pytest
from datetime import timedelta

from k1s0_file_client import (
    FileClientError,
    InMemoryFileClient,
)


async def test_generate_upload_url() -> None:
    client = InMemoryFileClient()
    url = await client.generate_upload_url("uploads/test.png", "image/png", timedelta(hours=1))
    assert "uploads/test.png" in url.url
    assert url.method == "PUT"


async def test_generate_download_url() -> None:
    client = InMemoryFileClient()
    await client.generate_upload_url("uploads/test.png", "image/png", timedelta(hours=1))
    url = await client.generate_download_url("uploads/test.png", timedelta(minutes=5))
    assert "uploads/test.png" in url.url
    assert url.method == "GET"


async def test_download_url_not_found() -> None:
    client = InMemoryFileClient()
    with pytest.raises(FileClientError):
        await client.generate_download_url("nonexistent.txt", timedelta(minutes=5))


async def test_delete() -> None:
    client = InMemoryFileClient()
    await client.generate_upload_url("uploads/test.png", "image/png", timedelta(hours=1))
    await client.delete("uploads/test.png")
    with pytest.raises(FileClientError):
        await client.get_metadata("uploads/test.png")


async def test_get_metadata() -> None:
    client = InMemoryFileClient()
    await client.generate_upload_url("uploads/test.png", "image/png", timedelta(hours=1))
    meta = await client.get_metadata("uploads/test.png")
    assert meta.path == "uploads/test.png"
    assert meta.content_type == "image/png"


async def test_list() -> None:
    client = InMemoryFileClient()
    await client.generate_upload_url("uploads/a.png", "image/png", timedelta(hours=1))
    await client.generate_upload_url("uploads/b.jpg", "image/jpeg", timedelta(hours=1))
    await client.generate_upload_url("other/c.txt", "text/plain", timedelta(hours=1))
    files = await client.list("uploads/")
    assert len(files) == 2


async def test_copy() -> None:
    client = InMemoryFileClient()
    await client.generate_upload_url("uploads/test.png", "image/png", timedelta(hours=1))
    await client.copy("uploads/test.png", "archive/test.png")
    meta = await client.get_metadata("archive/test.png")
    assert meta.content_type == "image/png"
    assert meta.path == "archive/test.png"


async def test_copy_not_found() -> None:
    client = InMemoryFileClient()
    with pytest.raises(FileClientError):
        await client.copy("nonexistent.txt", "dest.txt")


async def test_stored_files_empty() -> None:
    client = InMemoryFileClient()
    assert len(client.stored_files) == 0


async def test_stored_files_returns_copy() -> None:
    client = InMemoryFileClient()
    await client.generate_upload_url("test.txt", "text/plain", timedelta(hours=1))
    files1 = client.stored_files
    files2 = client.stored_files
    assert files1 is not files2
