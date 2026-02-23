package graphqlclient_test

import (
	"context"
	"testing"

	graphqlclient "github.com/k1s0-platform/system-library-go-graphql-client"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestExecute(t *testing.T) {
	c := graphqlclient.NewInMemoryGraphQlClient()
	ctx := context.Background()

	expected := map[string]any{"user": map[string]any{"id": "1", "name": "Alice"}}
	c.SetResponse("GetUser", expected)

	query := graphqlclient.GraphQlQuery{
		Query:         "query GetUser($id: ID!) { user(id: $id) { id name } }",
		Variables:     map[string]any{"id": "1"},
		OperationName: "GetUser",
	}
	resp, err := c.Execute(ctx, query, nil)
	require.NoError(t, err)
	require.NotNil(t, resp.Data)
	assert.Empty(t, resp.Errors)
}

func TestExecute_NotConfigured(t *testing.T) {
	c := graphqlclient.NewInMemoryGraphQlClient()
	ctx := context.Background()

	query := graphqlclient.GraphQlQuery{
		Query:         "query Unknown { unknown }",
		OperationName: "Unknown",
	}
	_, err := c.Execute(ctx, query, nil)
	require.Error(t, err)
}

func TestExecuteMutation(t *testing.T) {
	c := graphqlclient.NewInMemoryGraphQlClient()
	ctx := context.Background()

	expected := map[string]any{"createUser": map[string]any{"id": "2", "name": "Bob"}}
	c.SetResponse("CreateUser", expected)

	mutation := graphqlclient.GraphQlQuery{
		Query:         "mutation CreateUser($name: String!) { createUser(name: $name) { id name } }",
		Variables:     map[string]any{"name": "Bob"},
		OperationName: "CreateUser",
	}
	resp, err := c.ExecuteMutation(ctx, mutation, nil)
	require.NoError(t, err)
	require.NotNil(t, resp.Data)
	assert.Empty(t, resp.Errors)
}

func TestSetResponse_Overwrite(t *testing.T) {
	c := graphqlclient.NewInMemoryGraphQlClient()
	ctx := context.Background()

	c.SetResponse("Op", "first")
	c.SetResponse("Op", "second")

	query := graphqlclient.GraphQlQuery{Query: "query Op { op }", OperationName: "Op"}
	resp, err := c.Execute(ctx, query, nil)
	require.NoError(t, err)
	data := *resp.Data
	assert.Equal(t, "second", data)
}
