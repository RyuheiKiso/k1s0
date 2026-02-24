"""pagination library unit tests."""

import pytest

from k1s0_pagination import (
    CursorMeta,
    CursorRequest,
    PageRequest,
    PageResponse,
    PaginationMeta,
    PerPageValidationError,
    decode_cursor,
    encode_cursor,
    validate_per_page,
)


def test_page_response_create() -> None:
    req = PageRequest(page=0, per_page=10)
    resp = PageResponse.create(items=["a", "b"], total=25, req=req)
    assert resp.items == ["a", "b"]
    assert resp.total == 25
    assert resp.page == 0
    assert resp.per_page == 10
    assert resp.total_pages == 3


def test_page_response_exact_division() -> None:
    req = PageRequest(page=0, per_page=5)
    resp = PageResponse.create(items=[], total=10, req=req)
    assert resp.total_pages == 2


def test_page_response_zero_per_page() -> None:
    req = PageRequest(page=0, per_page=0)
    resp = PageResponse.create(items=[], total=10, req=req)
    assert resp.total_pages == 0


def test_page_response_zero_total() -> None:
    req = PageRequest(page=0, per_page=10)
    resp = PageResponse.create(items=[], total=0, req=req)
    assert resp.total_pages == 0


def test_page_response_meta() -> None:
    req = PageRequest(page=2, per_page=10)
    resp = PageResponse.create(items=["a"], total=25, req=req)
    meta = resp.meta
    assert isinstance(meta, PaginationMeta)
    assert meta.total == 25
    assert meta.page == 2
    assert meta.per_page == 10
    assert meta.total_pages == 3


def test_encode_decode_cursor_roundtrip() -> None:
    sort_key = "2024-01-15"
    id_ = "abc-123"
    cursor = encode_cursor(sort_key, id_)
    decoded_sort_key, decoded_id = decode_cursor(cursor)
    assert decoded_sort_key == sort_key
    assert decoded_id == id_


def test_decode_cursor_missing_separator() -> None:
    import base64

    bad_cursor = base64.b64encode(b"noseparator").decode()
    with pytest.raises(ValueError, match="missing separator"):
        decode_cursor(bad_cursor)


def test_cursor_request_fields() -> None:
    req = CursorRequest(cursor="abc", limit=20)
    assert req.cursor == "abc"
    assert req.limit == 20


def test_cursor_meta_fields() -> None:
    meta = CursorMeta(next_cursor="next", has_more=True)
    assert meta.next_cursor == "next"
    assert meta.has_more is True


def test_pagination_meta_fields() -> None:
    meta = PaginationMeta(total=100, page=2, per_page=10, total_pages=10)
    assert meta.total == 100
    assert meta.total_pages == 10


def test_validate_per_page_valid() -> None:
    assert validate_per_page(1) == 1
    assert validate_per_page(50) == 50
    assert validate_per_page(100) == 100


def test_validate_per_page_zero() -> None:
    with pytest.raises(PerPageValidationError):
        validate_per_page(0)


def test_validate_per_page_over_max() -> None:
    with pytest.raises(PerPageValidationError):
        validate_per_page(101)
