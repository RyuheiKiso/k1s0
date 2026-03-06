package graphqlclient

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"sync"
)

// ClientErrorKind はクライアントエラーの種別。
type ClientErrorKind int

const (
	// ClientErrorRequest はリクエストエラー。
	ClientErrorRequest ClientErrorKind = iota
	// ClientErrorDeserialization はデシリアライズエラー。
	ClientErrorDeserialization
	// ClientErrorGraphQl はGraphQLエラー。
	ClientErrorGraphQl
	// ClientErrorNotFound はリソース未検出エラー。
	ClientErrorNotFound
)

// String は ClientErrorKind の文字列表現を返す。
func (k ClientErrorKind) String() string {
	switch k {
	case ClientErrorRequest:
		return "request error"
	case ClientErrorDeserialization:
		return "deserialization error"
	case ClientErrorGraphQl:
		return "graphql error"
	case ClientErrorNotFound:
		return "not found"
	default:
		return "unknown error"
	}
}

// ClientError はGraphQLクライアントのエラー。
type ClientError struct {
	Kind    ClientErrorKind
	Message string
}

// Error は error インターフェースの実装。
func (e *ClientError) Error() string {
	return fmt.Sprintf("%s: %s", e.Kind, e.Message)
}

// NewRequestError は新しいリクエストエラーを生成する。
func NewRequestError(msg string) *ClientError {
	return &ClientError{Kind: ClientErrorRequest, Message: msg}
}

// NewDeserializationError は新しいデシリアライズエラーを生成する。
func NewDeserializationError(msg string) *ClientError {
	return &ClientError{Kind: ClientErrorDeserialization, Message: msg}
}

// NewGraphQlError は新しいGraphQLエラーを生成する。
func NewGraphQlError(msg string) *ClientError {
	return &ClientError{Kind: ClientErrorGraphQl, Message: msg}
}

// NewNotFoundError は新しいリソース未検出エラーを生成する。
func NewNotFoundError(msg string) *ClientError {
	return &ClientError{Kind: ClientErrorNotFound, Message: msg}
}

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
	Subscribe(ctx context.Context, subscription GraphQlQuery) (<-chan *GraphQlResponse[any], error)
}

// InMemoryGraphQlClient はメモリ内のGraphQLクライアント。
type InMemoryGraphQlClient struct {
	Responses     map[string]any
	subscriptions map[string][]any
	mu            sync.RWMutex
}

// NewInMemoryGraphQlClient は新しい InMemoryGraphQlClient を生成する。
func NewInMemoryGraphQlClient() *InMemoryGraphQlClient {
	return &InMemoryGraphQlClient{
		Responses:     make(map[string]any),
		subscriptions: make(map[string][]any),
	}
}

// SetResponse はオペレーション名に対するモック応答を設定する。
func (c *InMemoryGraphQlClient) SetResponse(operationName string, response any) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.Responses[operationName] = response
}

// SetSubscriptionEvents はオペレーション名に対するサブスクリプションイベント列を設定する。
func (c *InMemoryGraphQlClient) SetSubscriptionEvents(operationName string, events []any) {
	c.mu.Lock()
	defer c.mu.Unlock()
	if c.subscriptions == nil {
		c.subscriptions = make(map[string][]any)
	}
	c.subscriptions[operationName] = events
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

// Subscribe はGraphQLサブスクリプションを実行し、イベントチャンネルを返す。
func (c *InMemoryGraphQlClient) Subscribe(_ context.Context, subscription GraphQlQuery) (<-chan *GraphQlResponse[any], error) {
	key := subscription.OperationName
	if key == "" {
		key = subscription.Query
	}
	c.mu.RLock()
	events, ok := c.subscriptions[key]
	c.mu.RUnlock()
	if !ok {
		events = nil
	}
	ch := make(chan *GraphQlResponse[any], len(events))
	for _, event := range events {
		e := event
		ch <- &GraphQlResponse[any]{Data: &e}
	}
	close(ch)
	return ch, nil
}

// GraphQlHttpClient はHTTPベースのGraphQLクライアント。
type GraphQlHttpClient struct {
	endpoint   string
	headers    map[string]string
	httpClient *http.Client
}

// NewGraphQlHttpClient は新しい GraphQlHttpClient を生成する。
func NewGraphQlHttpClient(endpoint string, headers map[string]string) *GraphQlHttpClient {
	h := headers
	if h == nil {
		h = make(map[string]string)
	}
	return &GraphQlHttpClient{
		endpoint:   endpoint,
		headers:    h,
		httpClient: &http.Client{},
	}
}

// Execute はGraphQLクエリをHTTPで実行する。
func (c *GraphQlHttpClient) Execute(ctx context.Context, query GraphQlQuery, _ any) (*GraphQlResponse[any], error) {
	return c.send(ctx, query)
}

// ExecuteMutation はGraphQLミューテーションをHTTPで実行する。
func (c *GraphQlHttpClient) ExecuteMutation(ctx context.Context, mutation GraphQlQuery, _ any) (*GraphQlResponse[any], error) {
	return c.send(ctx, mutation)
}

// Subscribe はGraphQLサブスクリプションを実行する。HTTPでは非対応のためエラーを返す。
func (c *GraphQlHttpClient) Subscribe(_ context.Context, _ GraphQlQuery) (<-chan *GraphQlResponse[any], error) {
	return nil, NewRequestError("GraphQlHttpClient does not support subscriptions over HTTP")
}

func (c *GraphQlHttpClient) send(ctx context.Context, query GraphQlQuery) (*GraphQlResponse[any], error) {
	body, err := json.Marshal(query)
	if err != nil {
		return nil, NewRequestError(fmt.Sprintf("failed to marshal query: %s", err))
	}

	req, err := http.NewRequestWithContext(ctx, http.MethodPost, c.endpoint, bytes.NewReader(body))
	if err != nil {
		return nil, NewRequestError(fmt.Sprintf("failed to create request: %s", err))
	}
	req.Header.Set("Content-Type", "application/json")
	for k, v := range c.headers {
		req.Header.Set(k, v)
	}

	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, NewRequestError(err.Error())
	}
	defer resp.Body.Close()

	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, NewRequestError(fmt.Sprintf("failed to read response body: %s", err))
	}

	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		return nil, NewRequestError(fmt.Sprintf("status %d: %s", resp.StatusCode, string(respBody)))
	}

	var gqlResp GraphQlResponse[any]
	if err := json.Unmarshal(respBody, &gqlResp); err != nil {
		return nil, NewDeserializationError(err.Error())
	}

	return &gqlResp, nil
}
