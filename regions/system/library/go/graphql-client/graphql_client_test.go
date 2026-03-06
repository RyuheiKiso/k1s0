package graphqlclient_test

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
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

func TestInMemoryGraphQlClient_Subscribe(t *testing.T) {
	client := graphqlclient.NewInMemoryGraphQlClient()
	events := []any{
		map[string]any{"id": "1", "name": "Alice"},
		map[string]any{"id": "2", "name": "Bob"},
	}
	client.SetSubscriptionEvents("OnUserCreated", events)

	subscription := graphqlclient.GraphQlQuery{
		Query:         "subscription { userCreated { id name } }",
		OperationName: "OnUserCreated",
	}
	ch, err := client.Subscribe(context.Background(), subscription)
	assert.NoError(t, err)

	count := 0
	for resp := range ch {
		assert.NotNil(t, resp.Data)
		count++
	}
	assert.Equal(t, 2, count)
}

// --- ClientError tests ---

func TestClientError_Error(t *testing.T) {
	tests := []struct {
		err      *graphqlclient.ClientError
		expected string
	}{
		{graphqlclient.NewRequestError("timeout"), "request error: timeout"},
		{graphqlclient.NewDeserializationError("bad json"), "deserialization error: bad json"},
		{graphqlclient.NewGraphQlError("field not found"), "graphql error: field not found"},
		{graphqlclient.NewNotFoundError("user 123"), "not found: user 123"},
	}
	for _, tt := range tests {
		assert.Equal(t, tt.expected, tt.err.Error())
	}
}

func TestClientError_Kind(t *testing.T) {
	assert.Equal(t, graphqlclient.ClientErrorRequest, graphqlclient.NewRequestError("x").Kind)
	assert.Equal(t, graphqlclient.ClientErrorDeserialization, graphqlclient.NewDeserializationError("x").Kind)
	assert.Equal(t, graphqlclient.ClientErrorGraphQl, graphqlclient.NewGraphQlError("x").Kind)
	assert.Equal(t, graphqlclient.ClientErrorNotFound, graphqlclient.NewNotFoundError("x").Kind)
}

func TestClientError_ImplementsError(t *testing.T) {
	var err error = graphqlclient.NewRequestError("test")
	assert.Error(t, err)
}

// --- GraphQlHttpClient tests ---

func TestGraphQlHttpClient_Execute(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, http.MethodPost, r.Method)
		assert.Equal(t, "application/json", r.Header.Get("Content-Type"))

		var query graphqlclient.GraphQlQuery
		err := json.NewDecoder(r.Body).Decode(&query)
		require.NoError(t, err)
		assert.Equal(t, "query GetUser { user { id } }", query.Query)

		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write([]byte(`{"data":{"user":{"id":"1"}}}`))
	}))
	defer server.Close()

	client := graphqlclient.NewGraphQlHttpClient(server.URL, nil)
	query := graphqlclient.GraphQlQuery{
		Query:         "query GetUser { user { id } }",
		OperationName: "GetUser",
	}
	resp, err := client.Execute(context.Background(), query, nil)
	require.NoError(t, err)
	require.NotNil(t, resp.Data)
	assert.Empty(t, resp.Errors)
}

func TestGraphQlHttpClient_ExecuteMutation(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write([]byte(`{"data":{"createUser":{"id":"2"}}}`))
	}))
	defer server.Close()

	client := graphqlclient.NewGraphQlHttpClient(server.URL, nil)
	mutation := graphqlclient.GraphQlQuery{
		Query:         "mutation CreateUser { createUser { id } }",
		OperationName: "CreateUser",
	}
	resp, err := client.ExecuteMutation(context.Background(), mutation, nil)
	require.NoError(t, err)
	require.NotNil(t, resp.Data)
}

func TestGraphQlHttpClient_CustomHeaders(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, "Bearer token123", r.Header.Get("Authorization"))
		assert.Equal(t, "custom-value", r.Header.Get("X-Custom"))

		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write([]byte(`{"data":null}`))
	}))
	defer server.Close()

	headers := map[string]string{
		"Authorization": "Bearer token123",
		"X-Custom":      "custom-value",
	}
	client := graphqlclient.NewGraphQlHttpClient(server.URL, headers)
	query := graphqlclient.GraphQlQuery{Query: "{ ping }"}
	_, err := client.Execute(context.Background(), query, nil)
	require.NoError(t, err)
}

func TestGraphQlHttpClient_ServerError(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusInternalServerError)
		_, _ = w.Write([]byte("internal server error"))
	}))
	defer server.Close()

	client := graphqlclient.NewGraphQlHttpClient(server.URL, nil)
	query := graphqlclient.GraphQlQuery{Query: "{ ping }"}
	_, err := client.Execute(context.Background(), query, nil)
	require.Error(t, err)

	var clientErr *graphqlclient.ClientError
	require.ErrorAs(t, err, &clientErr)
	assert.Equal(t, graphqlclient.ClientErrorRequest, clientErr.Kind)
	assert.Contains(t, clientErr.Message, "500")
}

func TestGraphQlHttpClient_InvalidJSON(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write([]byte(`not json`))
	}))
	defer server.Close()

	client := graphqlclient.NewGraphQlHttpClient(server.URL, nil)
	query := graphqlclient.GraphQlQuery{Query: "{ ping }"}
	_, err := client.Execute(context.Background(), query, nil)
	require.Error(t, err)

	var clientErr *graphqlclient.ClientError
	require.ErrorAs(t, err, &clientErr)
	assert.Equal(t, graphqlclient.ClientErrorDeserialization, clientErr.Kind)
}

func TestGraphQlHttpClient_GraphQlErrors(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write([]byte(`{"data":null,"errors":[{"message":"field not found"}]}`))
	}))
	defer server.Close()

	client := graphqlclient.NewGraphQlHttpClient(server.URL, nil)
	query := graphqlclient.GraphQlQuery{Query: "{ unknown }"}
	resp, err := client.Execute(context.Background(), query, nil)
	require.NoError(t, err)
	assert.Len(t, resp.Errors, 1)
	assert.Equal(t, "field not found", resp.Errors[0].Message)
}

func TestGraphQlHttpClient_Subscribe_NotSupported(t *testing.T) {
	client := graphqlclient.NewGraphQlHttpClient("http://localhost", nil)
	ch, err := client.Subscribe(context.Background(), graphqlclient.GraphQlQuery{Query: "subscription { x }"})
	require.Error(t, err)
	assert.Nil(t, ch)

	var clientErr *graphqlclient.ClientError
	require.ErrorAs(t, err, &clientErr)
	assert.Equal(t, graphqlclient.ClientErrorRequest, clientErr.Kind)
	assert.Contains(t, clientErr.Message, "does not support subscriptions")
}

func TestGraphQlHttpClient_ConnectionRefused(t *testing.T) {
	client := graphqlclient.NewGraphQlHttpClient("http://127.0.0.1:1", nil)
	query := graphqlclient.GraphQlQuery{Query: "{ ping }"}
	_, err := client.Execute(context.Background(), query, nil)
	require.Error(t, err)

	var clientErr *graphqlclient.ClientError
	require.ErrorAs(t, err, &clientErr)
	assert.Equal(t, graphqlclient.ClientErrorRequest, clientErr.Kind)
}

func TestGraphQlHttpClient_ImplementsInterface(t *testing.T) {
	var _ graphqlclient.GraphQlClient = graphqlclient.NewGraphQlHttpClient("http://localhost", nil)
}
