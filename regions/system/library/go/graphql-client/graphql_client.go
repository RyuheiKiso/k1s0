package graphqlclient

import (
	"context"
	"fmt"
	"sync"
)

// GraphQlQuery はGraphQLクエリ。
type GraphQlQuery struct {
	Query         string         `json:"query"`
	Variables     map[string]any `json:"variables,omitempty"`
	OperationName string         `json:"operationName,omitempty"`
}

// GraphQlError はGraphQLエラー。
type GraphQlError struct {
	Message   string          `json:"message"`
	Locations []ErrorLocation `json:"locations,omitempty"`
	Path      []any           `json:"path,omitempty"`
}

// ErrorLocation はエラーの位置情報。
type ErrorLocation struct {
	Line   int `json:"line"`
	Column int `json:"column"`
}

// GraphQlResponse はGraphQLレスポンス。
type GraphQlResponse[T any] struct {
	Data   *T             `json:"data,omitempty"`
	Errors []GraphQlError `json:"errors,omitempty"`
}

// GraphQlClient はGraphQLクライアントのインターフェース。
type GraphQlClient interface {
	Execute(ctx context.Context, query GraphQlQuery, result any) (*GraphQlResponse[any], error)
	ExecuteMutation(ctx context.Context, mutation GraphQlQuery, result any) (*GraphQlResponse[any], error)
}

// InMemoryGraphQlClient はメモリ内のGraphQLクライアント。
type InMemoryGraphQlClient struct {
	Responses map[string]any
	mu        sync.RWMutex
}

// NewInMemoryGraphQlClient は新しい InMemoryGraphQlClient を生成する。
func NewInMemoryGraphQlClient() *InMemoryGraphQlClient {
	return &InMemoryGraphQlClient{
		Responses: make(map[string]any),
	}
}

// SetResponse はオペレーション名に対するモック応答を設定する。
func (c *InMemoryGraphQlClient) SetResponse(operationName string, response any) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.Responses[operationName] = response
}

// Execute はGraphQLクエリを実行する。
func (c *InMemoryGraphQlClient) Execute(_ context.Context, query GraphQlQuery, _ any) (*GraphQlResponse[any], error) {
	c.mu.RLock()
	defer c.mu.RUnlock()
	resp, ok := c.Responses[query.OperationName]
	if !ok {
		return nil, fmt.Errorf("no response configured for operation: %s", query.OperationName)
	}
	return &GraphQlResponse[any]{Data: &resp}, nil
}

// ExecuteMutation はGraphQLミューテーションを実行する。
func (c *InMemoryGraphQlClient) ExecuteMutation(ctx context.Context, mutation GraphQlQuery, result any) (*GraphQlResponse[any], error) {
	return c.Execute(ctx, mutation, result)
}
