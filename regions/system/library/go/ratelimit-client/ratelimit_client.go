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

// GrpcRateLimitClient は ratelimit-server への HTTP クライアント。
// 実際の gRPC プロトコルではなく HTTP REST API を使用する。
type GrpcRateLimitClient struct {
	baseURL    string
	httpClient *http.Client
}

// NewGrpcRateLimitClient は新しい GrpcRateLimitClient を生成する。
// addr には "host:port" または "http://host:port" 形式のサーバーアドレスを指定する。
func NewGrpcRateLimitClient(addr string) (*GrpcRateLimitClient, error) {
	base := addr
	if !strings.HasPrefix(base, "http://") && !strings.HasPrefix(base, "https://") {
		base = "http://" + base
	}
	base = strings.TrimRight(base, "/")
	return &GrpcRateLimitClient{
		baseURL:    base,
		httpClient: &http.Client{Timeout: 30 * time.Second},
	}, nil
}

// NewGrpcRateLimitClientWithHTTPClient はテスト用コンストラクタ。カスタム http.Client を注入できる。
func NewGrpcRateLimitClientWithHTTPClient(addr string, httpClient *http.Client) (*GrpcRateLimitClient, error) {
	base := addr
	if !strings.HasPrefix(base, "http://") && !strings.HasPrefix(base, "https://") {
		base = "http://" + base
	}
	base = strings.TrimRight(base, "/")
	return &GrpcRateLimitClient{
		baseURL:    base,
		httpClient: httpClient,
	}, nil
}

func (c *GrpcRateLimitClient) doRequest(ctx context.Context, method, path string, body interface{}) (*http.Response, error) {
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
type checkRequest struct {
	Cost uint32 `json:"cost"`
}

// checkResponse は Check エンドポイントのレスポンス。
type checkResponse struct {
	Allowed        bool    `json:"allowed"`
	Remaining      uint32  `json:"remaining"`
	ResetAt        string  `json:"reset_at"`
	RetryAfterSecs *uint64 `json:"retry_after_secs"`
}

// consumeRequest は Consume エンドポイントへのリクエスト本文。
type consumeRequest struct {
	Cost uint32 `json:"cost"`
}

// consumeResponse は Consume エンドポイントのレスポンス。
type consumeResponse struct {
	Remaining uint32 `json:"remaining"`
	ResetAt   string `json:"reset_at"`
}

// policyResponse は GetLimit エンドポイントのレスポンス。
type policyResponse struct {
	Key        string `json:"key"`
	Limit      uint32 `json:"limit"`
	WindowSecs uint64 `json:"window_secs"`
	Algorithm  string `json:"algorithm"`
}

// Check はレート制限をチェックする。
// POST /api/v1/ratelimit/{key}/check
func (c *GrpcRateLimitClient) Check(ctx context.Context, key string, cost uint32) (RateLimitStatus, error) {
	path := fmt.Sprintf("/api/v1/ratelimit/%s/check", key)
	resp, err := c.doRequest(ctx, http.MethodPost, path, checkRequest{Cost: cost})
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
		Remaining:      result.Remaining,
		ResetAt:        resetAt,
		RetryAfterSecs: result.RetryAfterSecs,
	}, nil
}

// Consume は使用量を消費する。
// POST /api/v1/ratelimit/{key}/consume
func (c *GrpcRateLimitClient) Consume(ctx context.Context, key string, cost uint32) (RateLimitResult, error) {
	path := fmt.Sprintf("/api/v1/ratelimit/%s/consume", key)
	resp, err := c.doRequest(ctx, http.MethodPost, path, consumeRequest{Cost: cost})
	if err != nil {
		return RateLimitResult{}, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return RateLimitResult{}, parseRateLimitError(resp, "consume")
	}

	var result consumeResponse
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return RateLimitResult{}, &RateLimitError{Code: "SERVER_ERROR", Message: fmt.Sprintf("consume: decode response: %s", err)}
	}

	resetAt, err := time.Parse(time.RFC3339, result.ResetAt)
	if err != nil {
		resetAt = time.Time{}
	}

	return RateLimitResult{
		Remaining: result.Remaining,
		ResetAt:   resetAt,
	}, nil
}

// GetLimit はキーに対する制限ポリシーを取得する。
// GET /api/v1/ratelimit/{key}/policy
func (c *GrpcRateLimitClient) GetLimit(ctx context.Context, key string) (RateLimitPolicy, error) {
	path := fmt.Sprintf("/api/v1/ratelimit/%s/policy", key)
	resp, err := c.doRequest(ctx, http.MethodGet, path, nil)
	if err != nil {
		return RateLimitPolicy{}, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return RateLimitPolicy{}, parseRateLimitError(resp, "get_limit")
	}

	var result policyResponse
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return RateLimitPolicy{}, &RateLimitError{Code: "SERVER_ERROR", Message: fmt.Sprintf("get_limit: decode response: %s", err)}
	}

	return RateLimitPolicy{
		Key:        result.Key,
		Limit:      result.Limit,
		WindowSecs: result.WindowSecs,
		Algorithm:  result.Algorithm,
	}, nil
}
