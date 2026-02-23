"""GraphQL types."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Generic, TypeVar

T = TypeVar("T")


@dataclass
class GraphQlQuery:
    """GraphQL query or mutation."""

    query: str
    variables: dict[str, Any] | None = None
    operation_name: str | None = None


@dataclass
class ErrorLocation:
    """Error location in a GraphQL document."""

    line: int
    column: int


@dataclass
class GraphQlError:
    """GraphQL error."""

    message: str
    locations: list[ErrorLocation] | None = None
    path: list[Any] | None = None


@dataclass
class GraphQlResponse(Generic[T]):
    """GraphQL response."""

    data: T | None = None
    errors: list[GraphQlError] | None = None

    @property
    def has_errors(self) -> bool:
        return bool(self.errors)
