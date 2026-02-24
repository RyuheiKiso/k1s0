package distributedlock

import (
	"context"
	"testing"
	"time"

	"github.com/redis/go-redis/v9"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestRedisLock_LockKey(t *testing.T) {
	l := NewRedisLock(nil)
	assert.Equal(t, "lock:mykey", l.lockKey("mykey"))
}

func TestRedisLock_LockKey_CustomPrefix(t *testing.T) {
	l := NewRedisLock(nil, WithLockPrefix("myapp:lock"))
	assert.Equal(t, "myapp:lock:mykey", l.lockKey("mykey"))
}

func TestNewRedisLockFromURL_InvalidURL(t *testing.T) {
	_, err := NewRedisLockFromURL("not-a-valid-url")
	require.Error(t, err)
}

func TestNewRedisLockFromURL_ValidURL(t *testing.T) {
	l, err := NewRedisLockFromURL("redis://localhost:6379/0")
	require.NoError(t, err)
	assert.NotNil(t, l)
}

func TestGenerateRedisToken(t *testing.T) {
	token1 := generateRedisToken()
	token2 := generateRedisToken()
	assert.Len(t, token1, 32)
	assert.Len(t, token2, 32)
	assert.NotEqual(t, token1, token2)
}

// Connection error tests verify proper error propagation from Redis.
func setupRedisLock(t *testing.T) (*RedisLock, func()) {
	t.Helper()
	rdb := redis.NewClient(&redis.Options{
		Addr: "localhost:0", // invalid port, won't connect
	})
	l := NewRedisLock(rdb)
	return l, func() { rdb.Close() }
}

func TestRedisLock_Acquire_ConnectionError(t *testing.T) {
	l, cleanup := setupRedisLock(t)
	defer cleanup()

	_, err := l.Acquire(context.Background(), "key1", time.Second)
	assert.Error(t, err)
}

func TestRedisLock_IsLocked_ConnectionError(t *testing.T) {
	l, cleanup := setupRedisLock(t)
	defer cleanup()

	_, err := l.IsLocked(context.Background(), "key1")
	assert.Error(t, err)
}
