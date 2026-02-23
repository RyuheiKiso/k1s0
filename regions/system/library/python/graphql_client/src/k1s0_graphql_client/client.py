"""GraphQL client abstraction."""

from __future__ import annotations

from abc import ABC, abstractmethod
from enum import Enum, auto
from typing import Any

from .types import GraphQlQuery, GraphQlResponse


class GraphQlClientError(Exception):
    """GraphQL client error."""

    class Code(Enum):
        OPERATION_NOT_FOUND = auto()
        UNKNOWN_OPERATION = auto()

    def __init__(self, message: str, code: GraphQlClientError.Code) -> None:
        super().__init__(message)
        self.code = code


class GraphQlClient(ABC):
    """Abstract GraphQL client."""

    @abstractmethod
    async def execute(self, query: GraphQlQuery) -> GraphQlResponse[Any]:
        ...

    @abstractmethod
    async def execute_mutation(self, mutation: GraphQlQuery) -> GraphQlResponse[Any]:
        ...


class InMemoryGraphQlClient(GraphQlClient):
    """In-memory GraphQL client for testing."""

    def __init__(self) -> None:
        self._responses: dict[str, Any] = {}

    def set_response(self, operation_name: str, response: Any) -> None:
        self._responses[operation_name] = response

    async def execute(self, query: GraphQlQuery) -> GraphQlResponse[Any]:
        return self._resolve(query)

    async def execute_mutation(self, mutation: GraphQlQuery) -> GraphQlResponse[Any]:
        return self._resolve(mutation)

    def _resolve(self, query: GraphQlQuery) -> GraphQlResponse[Any]:
        if query.operation_name is None:
            raise GraphQlClientError(
                "No operation name provided",
                GraphQlClientError.Code.UNKNOWN_OPERATION,
            )
        if query.operation_name not in self._responses:
            raise GraphQlClientError(
                f"Operation not found: {query.operation_name}",
                GraphQlClientError.Code.OPERATION_NOT_FOUND,
            )
        return self._responses[query.operation_name]
