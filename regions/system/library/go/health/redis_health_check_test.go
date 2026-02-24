package health

import (
	"context"
	"testing"
	"time"

	"github.com/redis/go-redis/v9"
	"github.com/stretchr/testify/assert"
)

func TestRedisHealthCheck_Name(t *testing.T) {
	h := NewRedisHealthCheck(nil)
	assert.Equal(t, "redis", h.Name())
}

func TestRedisHealthCheck_CustomName(t *testing.T) {
	h := NewRedisHealthCheck(nil, WithRedisName("cache-redis"))
	assert.Equal(t, "cache-redis", h.Name())
}

func TestRedisHealthCheck_DefaultTimeout(t *testing.T) {
	h := NewRedisHealthCheck(nil)
	assert.Equal(t, 5*time.Second, h.timeout)
}

func TestRedisHealthCheck_CustomTimeout(t *testing.T) {
	h := NewRedisHealthCheck(nil, WithRedisTimeout(2*time.Second))
	assert.Equal(t, 2*time.Second, h.timeout)
}

func TestRedisHealthCheck_ImplementsHealthCheck(t *testing.T) {
	var _ HealthCheck = (*RedisHealthCheck)(nil)
}

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
