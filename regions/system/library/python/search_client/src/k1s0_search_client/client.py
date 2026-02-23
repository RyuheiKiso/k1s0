"""Search client for indexing and searching documents."""

from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from typing import Any

from .exceptions import SearchError, SearchErrorCode


@dataclass
class Filter:
    """Search filter."""

    field: str
    operator: str  # "eq", "lt", "gt", "range", "in"
    value: Any
    value_to: Any = None

    @classmethod
    def eq(cls, field: str, value: Any) -> Filter:
        return cls(field=field, operator="eq", value=value)

    @classmethod
    def lt(cls, field: str, value: Any) -> Filter:
        return cls(field=field, operator="lt", value=value)

    @classmethod
    def gt(cls, field: str, value: Any) -> Filter:
        return cls(field=field, operator="gt", value=value)

    @classmethod
    def range(cls, field: str, from_val: Any, to_val: Any) -> Filter:
        return cls(field=field, operator="range", value=from_val, value_to=to_val)


@dataclass
class FacetBucket:
    """Facet aggregation bucket."""

    value: str
    count: int


@dataclass
class SearchQuery:
    """Search query parameters."""

    query: str
    filters: list[Filter] = field(default_factory=list)
    facets: list[str] = field(default_factory=list)
    page: int = 0
    size: int = 20


@dataclass
class SearchResult:
    """Search result."""

    hits: list[dict[str, Any]]
    total: int
    facets: dict[str, list[FacetBucket]]
    took_ms: int


@dataclass
class IndexDocument:
    """Document to index."""

    id: str
    fields: dict[str, Any] = field(default_factory=dict)


@dataclass
class IndexResult:
    """Index operation result."""

    id: str
    version: int


@dataclass
class BulkFailure:
    """Bulk index individual failure."""

    id: str
    error: str


@dataclass
class BulkResult:
    """Bulk index result."""

    success_count: int
    failed_count: int
    failures: list[BulkFailure] = field(default_factory=list)


@dataclass
class FieldMapping:
    """Field mapping definition."""

    field_type: str
    indexed: bool = True


@dataclass
class IndexMapping:
    """Index mapping."""

    fields: dict[str, FieldMapping] = field(default_factory=dict)

    def with_field(self, name: str, field_type: str) -> IndexMapping:
        new_fields = dict(self.fields)
        new_fields[name] = FieldMapping(field_type=field_type)
        return IndexMapping(fields=new_fields)


class SearchClient(ABC):
    """Abstract search client."""

    @abstractmethod
    async def index_document(self, index: str, doc: IndexDocument) -> IndexResult: ...

    @abstractmethod
    async def bulk_index(self, index: str, docs: list[IndexDocument]) -> BulkResult: ...

    @abstractmethod
    async def search(self, index: str, query: SearchQuery) -> SearchResult: ...

    @abstractmethod
    async def delete_document(self, index: str, id: str) -> None: ...

    @abstractmethod
    async def create_index(self, name: str, mapping: IndexMapping) -> None: ...


class InMemorySearchClient(SearchClient):
    """In-memory search client for testing."""

    def __init__(self) -> None:
        self._indexes: dict[str, list[IndexDocument]] = {}

    async def create_index(self, name: str, mapping: IndexMapping) -> None:
        self._indexes[name] = []

    async def index_document(self, index: str, doc: IndexDocument) -> IndexResult:
        if index not in self._indexes:
            raise SearchError(f"Index not found: {index}", SearchErrorCode.INDEX_NOT_FOUND)
        self._indexes[index].append(doc)
        return IndexResult(id=doc.id, version=len(self._indexes[index]))

    async def bulk_index(self, index: str, docs: list[IndexDocument]) -> BulkResult:
        if index not in self._indexes:
            raise SearchError(f"Index not found: {index}", SearchErrorCode.INDEX_NOT_FOUND)
        self._indexes[index].extend(docs)
        return BulkResult(success_count=len(docs), failed_count=0)

    async def search(self, index: str, query: SearchQuery) -> SearchResult:
        if index not in self._indexes:
            raise SearchError(f"Index not found: {index}", SearchErrorCode.INDEX_NOT_FOUND)

        docs = self._indexes[index]
        if query.query:
            filtered = [
                d for d in docs
                if any(
                    isinstance(v, str) and query.query in v
                    for v in d.fields.values()
                )
            ]
        else:
            filtered = list(docs)

        start = query.page * query.size
        end = start + query.size
        paged = filtered[start:end]

        hits = [{"id": d.id, **d.fields} for d in paged]

        facets: dict[str, list[FacetBucket]] = {}
        for f in query.facets:
            facets[f] = [FacetBucket(value="default", count=len(hits))]

        return SearchResult(hits=hits, total=len(filtered), facets=facets, took_ms=1)

    async def delete_document(self, index: str, id: str) -> None:
        if index in self._indexes:
            self._indexes[index] = [d for d in self._indexes[index] if d.id != id]

    def document_count(self, index: str) -> int:
        return len(self._indexes.get(index, []))
