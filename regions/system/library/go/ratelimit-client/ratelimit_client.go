package ratelimitclient

import (
	"context"
	"fmt"
	"sync"
	"time"
)

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

// GrpcRateLimitClient は gRPC 経由で ratelimit-server に接続するクライアント。
type GrpcRateLimitClient struct {
	serverAddr string
}

// NewGrpcRateLimitClient は新しい GrpcRateLimitClient を生成する。
// addr には "host:port" 形式のサーバーアドレスを指定する（例: "ratelimit-server:8080"）。
func NewGrpcRateLimitClient(addr string) (*GrpcRateLimitClient, error) {
	return &GrpcRateLimitClient{
		serverAddr: addr,
	}, nil
}

// Check はレート制限をチェックする。
func (c *GrpcRateLimitClient) Check(_ context.Context, _ string, _ uint32) (RateLimitStatus, error) {
	return RateLimitStatus{}, fmt.Errorf("gRPC client not yet connected")
}

// Consume は使用量を消費する。
func (c *GrpcRateLimitClient) Consume(_ context.Context, _ string, _ uint32) (RateLimitResult, error) {
	return RateLimitResult{}, fmt.Errorf("gRPC client not yet connected")
}

// GetLimit はキーに対する制限ポリシーを取得する。
func (c *GrpcRateLimitClient) GetLimit(_ context.Context, _ string) (RateLimitPolicy, error) {
	return RateLimitPolicy{}, fmt.Errorf("gRPC client not yet connected")
}
