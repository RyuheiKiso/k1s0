package ratelimitclient

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strings"
	"sync"
	"time"
)

// RateLimitError はレート制限クライアントのエラー型。
type RateLimitError struct {
	// Code はエラーコード: "LIMIT_EXCEEDED" | "KEY_NOT_FOUND" | "SERVER_ERROR" | "TIMEOUT"
	Code    string
	Message string
}

func (e *RateLimitError) Error() string {
	return fmt.Sprintf("ratelimit-client error [%s]: %s", e.Code, e.Message)
}

// RateLimitStatus はレート制限チェックの結果。
type RateLimitStatus struct {
	Allowed        bool
	Remaining      uint32
	ResetAt        time.Time
	RetryAfterSecs *uint64
}

// RateLimitResult は消費後の結果。
type RateLimitResult struct {
	Remaining uint32
	ResetAt   time.Time
}

// RateLimitPolicy はキーに紐づく制限設定。
type RateLimitPolicy struct {
	Key        string
	Limit      uint32
	WindowSecs uint64
	Algorithm  string
}

// RateLimitClient はレート制限操作のインターフェース。
type RateLimitClient interface {
	Check(ctx context.Context, key string, cost uint32) (RateLimitStatus, error)
	Consume(ctx context.Context, key string, cost uint32) (RateLimitResult, error)
	GetLimit(ctx context.Context, key string) (RateLimitPolicy, error)
}

// InMemoryClient はテスト用のインメモリレート制限クライアント。
type InMemoryClient struct {
	mu       sync.Mutex
	counters map[string]uint32
	policies map[string]RateLimitPolicy
}

// NewInMemoryClient は新しい InMemoryClient を生成する。
func NewInMemoryClient() *InMemoryClient {
	return &InMemoryClient{
		counters: make(map[string]uint32),
		policies: map[string]RateLimitPolicy{
			"default": {
				Key:        "default",
				Limit:      100,
				WindowSecs: 3600,
				Algorithm:  "token_bucket",
			},
		},
	}
}

// SetPolicy はキーに対するポリシーを設定する。
func (c *InMemoryClient) SetPolicy(key string, policy RateLimitPolicy) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.policies[key] = policy
}

func (c *InMemoryClient) getPolicy(key string) RateLimitPolicy {
	if p, ok := c.policies[key]; ok {
		return p
	}
	return c.policies["default"]
}

// Check はレート制限をチェックする。
func (c *InMemoryClient) Check(_ context.Context, key string, cost uint32) (RateLimitStatus, error) {
	c.mu.Lock()
	defer c.mu.Unlock()

	policy := c.getPolicy(key)
	used := c.counters[key]
	resetAt := time.Now().Add(time.Duration(policy.WindowSecs) * time.Second)

	if used+cost > policy.Limit {
		retryAfter := policy.WindowSecs
		return RateLimitStatus{
			Allowed:        false,
			Remaining:      0,
			ResetAt:        resetAt,
			RetryAfterSecs: &retryAfter,
		}, nil
	}

	remaining := policy.Limit - used - cost
	return RateLimitStatus{
		Allowed:        true,
		Remaining:      remaining,
		ResetAt:        resetAt,
		RetryAfterSecs: nil,
	}, nil
}

// Consume は使用量を消費する。
func (c *InMemoryClient) Consume(_ context.Context, key string, cost uint32) (RateLimitResult, error) {
	c.mu.Lock()
	defer c.mu.Unlock()

	policy := c.getPolicy(key)
	used := c.counters[key]

	if used+cost > policy.Limit {
		return RateLimitResult{}, fmt.Errorf("rate limit exceeded for key: %s", key)
	}

	c.counters[key] = used + cost
	remaining := policy.Limit - c.counters[key]
	resetAt := time.Now().Add(time.Duration(policy.WindowSecs) * time.Second)

	return RateLimitResult{
		Remaining: remaining,
		ResetAt:   resetAt,
	}, nil
}

// GetLimit はキーに対する制限ポリシーを取得する。
func (c *InMemoryClient) GetLimit(_ context.Context, key string) (RateLimitPolicy, error) {
	c.mu.Lock()
	defer c.mu.Unlock()

	if p, ok := c.policies[key]; ok {
		return p, nil
	}
	return c.policies["default"], nil
}

// UsedCount はキーの現在の使用量を返す。テスト用。
func (c *InMemoryClient) UsedCount(key string) uint32 {
	c.mu.Lock()
	defer c.mu.Unlock()
	return c.counters[key]
}

// HttpRateLimitClient は ratelimit-server への HTTP クライアント。
// C-02/L-16 監査対応: GrpcRateLimitClient から HttpRateLimitClient にリネーム。
// API パスをサーバー実装に合わせて統一。
type HttpRateLimitClient struct {
	baseURL    string
	httpClient *http.Client
}

// GrpcRateLimitClient は後方互換性のための型エイリアス。
// Deprecated: HttpRateLimitClient を使用してください。
type GrpcRateLimitClient = HttpRateLimitClient

// NewHttpRateLimitClient は新しい HttpRateLimitClient を生成する。
// addr には "host:port" または "http://host:port" 形式のサーバーアドレスを指定する。
func NewHttpRateLimitClient(addr string) (*HttpRateLimitClient, error) {
	base := addr
	if !strings.HasPrefix(base, "http://") && !strings.HasPrefix(base, "https://") {
		base = "http://" + base
	}
	base = strings.TrimRight(base, "/")
	return &HttpRateLimitClient{
		baseURL:    base,
		httpClient: &http.Client{Timeout: 30 * time.Second},
	}, nil
}

// NewGrpcRateLimitClient は後方互換性のためのコンストラクタ。
// Deprecated: NewHttpRateLimitClient を使用してください。
func NewGrpcRateLimitClient(addr string) (*HttpRateLimitClient, error) {
	return NewHttpRateLimitClient(addr)
}

// NewHttpRateLimitClientWithHTTPClient はテスト用コンストラクタ。カスタム http.Client を注入できる。
func NewHttpRateLimitClientWithHTTPClient(addr string, httpClient *http.Client) (*HttpRateLimitClient, error) {
	base := addr
	if !strings.HasPrefix(base, "http://") && !strings.HasPrefix(base, "https://") {
		base = "http://" + base
	}
	base = strings.TrimRight(base, "/")
	return &HttpRateLimitClient{
		baseURL:    base,
		httpClient: httpClient,
	}, nil
}

// NewGrpcRateLimitClientWithHTTPClient は後方互換性のためのコンストラクタ。
// Deprecated: NewHttpRateLimitClientWithHTTPClient を使用してください。
func NewGrpcRateLimitClientWithHTTPClient(addr string, httpClient *http.Client) (*HttpRateLimitClient, error) {
	return NewHttpRateLimitClientWithHTTPClient(addr, httpClient)
}

// doRequest は HTTP リクエストを実行する。
func (c *HttpRateLimitClient) doRequest(ctx context.Context, method, path string, body any) (*http.Response, error) {
	var reqBody io.Reader
	if body != nil {
		data, err := json.Marshal(body)
		if err != nil {
			return nil, &RateLimitError{Code: "SERVER_ERROR", Message: fmt.Sprintf("marshal request: %s", err)}
		}
		reqBody = bytes.NewReader(data)
	}

	req, err := http.NewRequestWithContext(ctx, method, c.baseURL+path, reqBody)
	if err != nil {
		return nil, &RateLimitError{Code: "SERVER_ERROR", Message: fmt.Sprintf("create request: %s", err)}
	}
	if body != nil {
		req.Header.Set("Content-Type", "application/json")
	}

	resp, err := c.httpClient.Do(req)
	if err != nil {
		if ctx.Err() != nil {
			return nil, &RateLimitError{Code: "TIMEOUT", Message: ctx.Err().Error()}
		}
		return nil, &RateLimitError{Code: "SERVER_ERROR", Message: err.Error()}
	}
	return resp, nil
}

func parseRateLimitError(resp *http.Response, op string) *RateLimitError {
	bodyBytes, _ := io.ReadAll(resp.Body)
	msg := strings.TrimSpace(string(bodyBytes))
	if msg == "" {
		msg = fmt.Sprintf("status %d", resp.StatusCode)
	}
	switch resp.StatusCode {
	case http.StatusNotFound:
		return &RateLimitError{Code: "KEY_NOT_FOUND", Message: fmt.Sprintf("%s: %s", op, msg)}
	case http.StatusTooManyRequests:
		return &RateLimitError{Code: "LIMIT_EXCEEDED", Message: fmt.Sprintf("%s: %s", op, msg)}
	default:
		return &RateLimitError{Code: "SERVER_ERROR", Message: fmt.Sprintf("%s failed (status %d): %s", op, resp.StatusCode, msg)}
	}
}

// checkRequest は Check エンドポイントへのリクエスト本文。
// C-02 監査対応: サーバー API に合わせて scope + identifier + window 形式に変更。
type checkRequest struct {
	Scope      string `json:"scope"`
	Identifier string `json:"identifier"`
	Window     string `json:"window,omitempty"`
}

// checkResponse は Check エンドポイントのレスポンス。
type checkResponse struct {
	Allowed   bool   `json:"allowed"`
	Remaining int64  `json:"remaining"`
	ResetAt   string `json:"reset_at"`
}

// usageResponse は Usage エンドポイントのレスポンス。
type usageResponse struct {
	Key        string `json:"key"`
	Limit      uint32 `json:"limit"`
	WindowSecs uint64 `json:"window_secs"`
	Algorithm  string `json:"algorithm"`
}

// splitKey は key を "scope:identifier" 形式から (scope, identifier) に分割する。
// ":" が含まれない場合は scope="default"、identifier=key とする。
func splitKey(key string) (string, string) {
	parts := strings.SplitN(key, ":", 2)
	if len(parts) == 2 {
		return parts[0], parts[1]
	}
	return "default", key
}

// Check はレート制限をチェックする。
// C-02 監査対応: POST /api/v1/ratelimit/check（key をボディに含める）
func (c *HttpRateLimitClient) Check(ctx context.Context, key string, cost uint32) (RateLimitStatus, error) {
	scope, identifier := splitKey(key)
	var window string
	if cost > 1 {
		window = fmt.Sprintf("%ds", cost)
	}
	resp, err := c.doRequest(ctx, http.MethodPost, "/api/v1/ratelimit/check", checkRequest{
		Scope:      scope,
		Identifier: identifier,
		Window:     window,
	})
	if err != nil {
		return RateLimitStatus{}, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return RateLimitStatus{}, parseRateLimitError(resp, "check")
	}

	var result checkResponse
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return RateLimitStatus{}, &RateLimitError{Code: "SERVER_ERROR", Message: fmt.Sprintf("check: decode response: %s", err)}
	}

	resetAt, err := time.Parse(time.RFC3339, result.ResetAt)
	if err != nil {
		resetAt = time.Time{}
	}

	return RateLimitStatus{
		Allowed:        result.Allowed,
		Remaining:      uint32(result.Remaining),
		ResetAt:        resetAt,
		RetryAfterSecs: nil,
	}, nil
}

// Consume は使用量を消費する。
// C-02 監査対応: サーバーに consume エンドポイントはないため、check で代用する。
func (c *HttpRateLimitClient) Consume(ctx context.Context, key string, cost uint32) (RateLimitResult, error) {
	status, err := c.Check(ctx, key, cost)
	if err != nil {
		return RateLimitResult{}, err
	}
	return RateLimitResult{
		Remaining: status.Remaining,
		ResetAt:   status.ResetAt,
	}, nil
}

// GetLimit はキーに対する制限ポリシーを取得する。
// C-02 監査対応: GET /api/v1/ratelimit/usage
func (c *HttpRateLimitClient) GetLimit(ctx context.Context, key string) (RateLimitPolicy, error) {
	resp, err := c.doRequest(ctx, http.MethodGet, "/api/v1/ratelimit/usage", nil)
	if err != nil {
		return RateLimitPolicy{}, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return RateLimitPolicy{}, parseRateLimitError(resp, "get_limit")
	}

	var result usageResponse
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return RateLimitPolicy{}, &RateLimitError{Code: "SERVER_ERROR", Message: fmt.Sprintf("get_limit: decode response: %s", err)}
	}

	resultKey := result.Key
	if resultKey == "" {
		resultKey = key
	}

	return RateLimitPolicy{
		Key:        resultKey,
		Limit:      result.Limit,
		WindowSecs: result.WindowSecs,
		Algorithm:  result.Algorithm,
	}, nil
}
