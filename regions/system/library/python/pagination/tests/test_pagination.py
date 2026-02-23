"""pagination library unit tests."""

from k1s0_pagination import PageRequest, PageResponse, decode_cursor, encode_cursor


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


def test_encode_cursor() -> None:
    cursor = encode_cursor("abc-123")
    assert isinstance(cursor, str)
    assert cursor != "abc-123"


def test_decode_cursor() -> None:
    cursor = encode_cursor("abc-123")
    assert decode_cursor(cursor) == "abc-123"


def test_cursor_roundtrip() -> None:
    original = "550e8400-e29b-41d4-a716-446655440000"
    assert decode_cursor(encode_cursor(original)) == original
