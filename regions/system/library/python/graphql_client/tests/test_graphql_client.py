"""graphql_client library unit tests."""

import pytest

from k1s0_graphql_client import (
    ErrorLocation,
    GraphQlClient,
    GraphQlClientError,
    GraphQlError,
    GraphQlQuery,
    GraphQlResponse,
    InMemoryGraphQlClient,
)


async def test_execute_query() -> None:
    client = InMemoryGraphQlClient()
    response = GraphQlResponse(data={"name": "test"})
    client.set_response("GetUser", response)

    query = GraphQlQuery(query="{ user { name } }", operation_name="GetUser")
    result = await client.execute(query)
    assert result.data == {"name": "test"}
    assert result.has_errors is False


async def test_execute_mutation() -> None:
    client = InMemoryGraphQlClient()
    response = GraphQlResponse(data={"id": "123"})
    client.set_response("CreateUser", response)

    mutation = GraphQlQuery(
        query="mutation { createUser { id } }", operation_name="CreateUser"
    )
    result = await client.execute_mutation(mutation)
    assert result.data == {"id": "123"}


async def test_operation_not_found() -> None:
    client = InMemoryGraphQlClient()
    query = GraphQlQuery(query="{ user { name } }", operation_name="Unknown")
    with pytest.raises(GraphQlClientError) as exc_info:
        await client.execute(query)
    assert exc_info.value.code == GraphQlClientError.Code.OPERATION_NOT_FOUND


async def test_unknown_operation() -> None:
    client = InMemoryGraphQlClient()
    query = GraphQlQuery(query="{ user { name } }")
    with pytest.raises(GraphQlClientError) as exc_info:
        await client.execute(query)
    assert exc_info.value.code == GraphQlClientError.Code.UNKNOWN_OPERATION


async def test_error_response() -> None:
    client = InMemoryGraphQlClient()
    error = GraphQlError(
        message="Not found",
        locations=[ErrorLocation(line=1, column=5)],
        path=["user"],
    )
    response = GraphQlResponse(data=None, errors=[error])
    client.set_response("GetUser", response)

    query = GraphQlQuery(query="{ user { name } }", operation_name="GetUser")
    result = await client.execute(query)
    assert result.has_errors is True
    assert result.errors[0].message == "Not found"
    assert result.errors[0].locations[0].line == 1
    assert result.errors[0].path[0] == "user"


async def test_query_with_variables() -> None:
    query = GraphQlQuery(
        query="query($id: ID!) { user(id: $id) { name } }",
        variables={"id": "123"},
        operation_name="GetUser",
    )
    assert query.variables == {"id": "123"}
    assert query.operation_name == "GetUser"


async def test_response_no_errors() -> None:
    response = GraphQlResponse(data={"name": "test"})
    assert response.has_errors is False


async def test_response_empty_errors() -> None:
    response = GraphQlResponse(data={"name": "test"}, errors=[])
    assert response.has_errors is False


async def test_graphql_error_dataclass() -> None:
    error = GraphQlError(message="test error")
    assert error.message == "test error"
    assert error.locations is None
    assert error.path is None


async def test_error_location_dataclass() -> None:
    loc = ErrorLocation(line=3, column=10)
    assert loc.line == 3
    assert loc.column == 10
