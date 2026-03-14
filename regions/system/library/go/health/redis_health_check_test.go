package health

import (
	"context"
	"testing"
	"time"

	"github.com/redis/go-redis/v9"
	"github.com/stretchr/testify/assert"
)

// RedisHealthCheck_Nameがデフォルト名「redis」を返すことを検証する。
func TestRedisHealthCheck_Name(t *testing.T) {
	h := NewRedisHealthCheck(nil)
	assert.Equal(t, "redis", h.Name())
}

// RedisHealthCheck_CustomNameがWithRedisNameオプションでカスタム名を設定できることを検証する。
func TestRedisHealthCheck_CustomName(t *testing.T) {
	h := NewRedisHealthCheck(nil, WithRedisName("cache-redis"))
	assert.Equal(t, "cache-redis", h.Name())
}

// RedisHealthCheck_DefaultTimeoutがデフォルトタイムアウトとして5秒が設定されることを検証する。
func TestRedisHealthCheck_DefaultTimeout(t *testing.T) {
	h := NewRedisHealthCheck(nil)
	assert.Equal(t, 5*time.Second, h.timeout)
}

// RedisHealthCheck_CustomTimeoutがWithRedisTimeoutオプションでカスタムタイムアウトを設定できることを検証する。
func TestRedisHealthCheck_CustomTimeout(t *testing.T) {
	h := NewRedisHealthCheck(nil, WithRedisTimeout(2*time.Second))
	assert.Equal(t, 2*time.Second, h.timeout)
}

// RedisHealthCheck_ImplementsHealthCheckがRedisHealthCheckがHealthCheckインターフェースを実装していることを検証する。
func TestRedisHealthCheck_ImplementsHealthCheck(t *testing.T) {
	var _ HealthCheck = (*RedisHealthCheck)(nil)
}

// RedisHealthCheck_Check_ConnectionErrorが無効なアドレスへの接続失敗時にエラーを返すことを検証する。
func TestRedisHealthCheck_Check_ConnectionError(t *testing.T) {
	rdb := redis.NewClient(&redis.Options{
		Addr: "localhost:0", // invalid port
	})
	defer rdb.Close()

	h := NewRedisHealthCheck(rdb, WithRedisTimeout(100*time.Millisecond))
	err := h.Check(context.Background())
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "redis health check failed")
}
