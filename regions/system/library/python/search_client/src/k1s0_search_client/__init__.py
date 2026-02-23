"""k1s0 search client library."""

from .client import (
    BulkFailure,
    BulkResult,
    FacetBucket,
    FieldMapping,
    Filter,
    InMemorySearchClient,
    IndexDocument,
    IndexMapping,
    IndexResult,
    SearchClient,
    SearchQuery,
    SearchResult,
)
from .exceptions import SearchError, SearchErrorCode

__all__ = [
    "BulkFailure",
    "BulkResult",
    "FacetBucket",
    "FieldMapping",
    "Filter",
    "InMemorySearchClient",
    "IndexDocument",
    "IndexMapping",
    "IndexResult",
    "SearchClient",
    "SearchError",
    "SearchErrorCode",
    "SearchQuery",
    "SearchResult",
]
