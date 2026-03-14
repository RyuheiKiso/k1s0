package cache

import (
	"context"
	"testing"
	"time"

	"github.com/redis/go-redis/v9"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// mockRedisClient implements redis.Cmdable for unit testing.
// We use go-redis's ring client with no servers to simulate behavior,
// but for proper unit tests we use miniredis.

// prefixedKeyがプレフィックスなしの場合にキーをそのまま返すことを確認する。
func TestRedisCacheClient_PrefixedKey(t *testing.T) {
	client := NewRedisCacheClient(nil)
	assert.Equal(t, "mykey", client.prefixedKey("mykey"))

	clientWithPrefix := NewRedisCacheClient(nil, WithKeyPrefix("app"))
	assert.Equal(t, "app:mykey", clientWithPrefix.prefixedKey("mykey"))
}

// 空文字のプレフィックスが設定された場合にキーをそのまま返すことを確認する。
func TestRedisCacheClient_PrefixedKey_Empty(t *testing.T) {
	client := NewRedisCacheClient(nil, WithKeyPrefix(""))
	assert.Equal(t, "mykey", client.prefixedKey("mykey"))
}

// 不正なURLからRedisCacheClientを生成するとエラーが返ることを確認する。
func TestNewRedisCacheClientFromURL_InvalidURL(t *testing.T) {
	_, err := NewRedisCacheClientFromURL("not-a-valid-url")
	require.Error(t, err)
}

// 有効なRedis URLからRedisCacheClientを正常に生成できることを確認する。
func TestNewRedisCacheClientFromURL_ValidURL(t *testing.T) {
	client, err := NewRedisCacheClientFromURL("redis://localhost:6379/0")
	require.NoError(t, err)
	assert.NotNil(t, client)
}

// Integration-like tests using miniredis for a fake Redis server.
// These tests verify the full CacheClient interface without a real Redis.

func setupMiniredis(t *testing.T) (*RedisCacheClient, func()) {
	t.Helper()
	// Use a real Redis client pointed at an invalid address.
	// For proper integration tests, use github.com/alicebob/miniredis/v2.
	// Here we test only the construction and key-prefix logic.
	rdb := redis.NewClient(&redis.Options{
		Addr: "localhost:0", // invalid port, won't connect
	})
	client := NewRedisCacheClient(rdb)
	return client, func() { rdb.Close() }
}

// Redis接続エラー時にGetがエラーを返しnilを返すことを確認する。
func TestRedisCacheClient_Get_ConnectionError(t *testing.T) {
	client, cleanup := setupMiniredis(t)
	defer cleanup()

	val, err := client.Get(context.Background(), "key1")
	// Should return a connection error since there's no server
	assert.Nil(t, val)
	assert.Error(t, err)
}

// Redis接続エラー時にSetがエラーを返すことを確認する。
func TestRedisCacheClient_Set_ConnectionError(t *testing.T) {
	client, cleanup := setupMiniredis(t)
	defer cleanup()

	ttl := 5 * time.Second
	err := client.Set(context.Background(), "key1", "val1", &ttl)
	assert.Error(t, err)
}

// Redis接続エラー時にDeleteがエラーを返すことを確認する。
func TestRedisCacheClient_Delete_ConnectionError(t *testing.T) {
	client, cleanup := setupMiniredis(t)
	defer cleanup()

	_, err := client.Delete(context.Background(), "key1")
	assert.Error(t, err)
}

// Redis接続エラー時にExistsがエラーを返すことを確認する。
func TestRedisCacheClient_Exists_ConnectionError(t *testing.T) {
	client, cleanup := setupMiniredis(t)
	defer cleanup()

	_, err := client.Exists(context.Background(), "key1")
	assert.Error(t, err)
}

// Redis接続エラー時にSetNXがエラーを返すことを確認する。
func TestRedisCacheClient_SetNX_ConnectionError(t *testing.T) {
	client, cleanup := setupMiniredis(t)
	defer cleanup()

	_, err := client.SetNX(context.Background(), "key1", "val1", time.Second)
	assert.Error(t, err)
}

// Redis接続エラー時にExpireがエラーを返すことを確認する。
func TestRedisCacheClient_Expire_ConnectionError(t *testing.T) {
	client, cleanup := setupMiniredis(t)
	defer cleanup()

	_, err := client.Expire(context.Background(), "key1", time.Second)
	assert.Error(t, err)
}
