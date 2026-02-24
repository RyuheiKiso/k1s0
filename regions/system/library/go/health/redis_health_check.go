package health

import (
	"context"
	"fmt"
	"time"

	"github.com/redis/go-redis/v9"
)

// RedisHealthCheck は Redis のヘルスを確認する。
type RedisHealthCheck struct {
	name    string
	client  redis.Cmdable
	timeout time.Duration
}

// RedisHealthCheckOption は RedisHealthCheck の設定オプション。
type RedisHealthCheckOption func(*RedisHealthCheck)

// WithRedisTimeout はタイムアウトを設定する。
func WithRedisTimeout(d time.Duration) RedisHealthCheckOption {
	return func(h *RedisHealthCheck) {
		h.timeout = d
	}
}

// WithRedisName はヘルスチェック名を設定する。
func WithRedisName(name string) RedisHealthCheckOption {
	return func(h *RedisHealthCheck) {
		h.name = name
	}
}

// NewRedisHealthCheck は新しい RedisHealthCheck を生成する。
func NewRedisHealthCheck(client redis.Cmdable, opts ...RedisHealthCheckOption) *RedisHealthCheck {
	h := &RedisHealthCheck{
		name:    "redis",
		client:  client,
		timeout: 5 * time.Second,
	}
	for _, opt := range opts {
		opt(h)
	}
	return h
}

// Name はヘルスチェック名を返す。
func (h *RedisHealthCheck) Name() string {
	return h.name
}

// Check は Redis に対して PING を実行する。
func (h *RedisHealthCheck) Check(ctx context.Context) error {
	checkCtx, cancel := context.WithTimeout(ctx, h.timeout)
	defer cancel()

	result := h.client.Ping(checkCtx)
	if err := result.Err(); err != nil {
		return fmt.Errorf("redis health check failed: %w", err)
	}
	return nil
}
