"""search_client library unit tests."""

import pytest

from k1s0_search_client import (
    BulkFailure,
    FacetBucket,
    Filter,
    InMemorySearchClient,
    IndexDocument,
    IndexMapping,
    IndexResult,
    SearchError,
    SearchErrorCode,
    SearchQuery,
)


async def test_create_index_and_index_document() -> None:
    client = InMemorySearchClient()
    await client.create_index("products", IndexMapping())

    doc = IndexDocument(id="p-1", fields={"name": "Rust Programming"})
    result = await client.index_document("products", doc)
    assert result.id == "p-1"
    assert result.version == 1


async def test_index_document_missing_index() -> None:
    client = InMemorySearchClient()
    doc = IndexDocument(id="1", fields={})
    with pytest.raises(SearchError) as exc_info:
        await client.index_document("nonexistent", doc)
    assert exc_info.value.code == SearchErrorCode.INDEX_NOT_FOUND


async def test_bulk_index() -> None:
    client = InMemorySearchClient()
    await client.create_index("items", IndexMapping())

    docs = [
        IndexDocument(id="i-1", fields={"name": "Item 1"}),
        IndexDocument(id="i-2", fields={"name": "Item 2"}),
    ]
    result = await client.bulk_index("items", docs)
    assert result.success_count == 2
    assert result.failed_count == 0
    assert result.failures == []


async def test_search() -> None:
    client = InMemorySearchClient()
    await client.create_index("products", IndexMapping())
    await client.index_document(
        "products", IndexDocument(id="p-1", fields={"name": "Rust Programming"})
    )
    await client.index_document(
        "products", IndexDocument(id="p-2", fields={"name": "Go Language"})
    )

    query = SearchQuery(query="Rust", facets=["name"])
    result = await client.search("products", query)
    assert result.total == 1
    assert len(result.hits) == 1
    assert "name" in result.facets


async def test_search_missing_index() -> None:
    client = InMemorySearchClient()
    with pytest.raises(SearchError):
        await client.search("nonexistent", SearchQuery(query="test"))


async def test_delete_document() -> None:
    client = InMemorySearchClient()
    await client.create_index("products", IndexMapping())
    await client.index_document(
        "products", IndexDocument(id="p-1", fields={"name": "Test"})
    )
    await client.delete_document("products", "p-1")
    assert client.document_count("products") == 0


async def test_search_empty_query() -> None:
    client = InMemorySearchClient()
    await client.create_index("items", IndexMapping())
    await client.index_document(
        "items", IndexDocument(id="i-1", fields={"name": "Item"})
    )
    result = await client.search("items", SearchQuery(query=""))
    assert result.total == 1


async def test_filter_factories() -> None:
    eq = Filter.eq("status", "active")
    assert eq.operator == "eq"
    assert eq.field == "status"

    lt = Filter.lt("price", 100)
    assert lt.operator == "lt"

    gt = Filter.gt("price", 50)
    assert gt.operator == "gt"

    r = Filter.range("price", 10, 100)
    assert r.operator == "range"
    assert r.value_to == 100


async def test_index_mapping_with_field() -> None:
    mapping = IndexMapping().with_field("name", "text").with_field("price", "integer")
    assert len(mapping.fields) == 2
    assert mapping.fields["name"].field_type == "text"
    assert mapping.fields["name"].indexed is True


async def test_search_error_code() -> None:
    err = SearchError("test", SearchErrorCode.INDEX_NOT_FOUND)
    assert err.code == SearchErrorCode.INDEX_NOT_FOUND
    assert str(err) == "test"


async def test_bulk_failure_dataclass() -> None:
    failure = BulkFailure(id="doc-1", error="mapping error")
    assert failure.id == "doc-1"
    assert failure.error == "mapping error"


async def test_facet_bucket_dataclass() -> None:
    bucket = FacetBucket(value="books", count=42)
    assert bucket.value == "books"
    assert bucket.count == 42


async def test_document_count() -> None:
    client = InMemorySearchClient()
    assert client.document_count("nonexistent") == 0
    await client.create_index("test", IndexMapping())
    assert client.document_count("test") == 0
