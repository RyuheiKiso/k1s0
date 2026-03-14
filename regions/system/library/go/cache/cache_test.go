package cache_test

import (
	"context"
	"testing"
	"time"

	"github.com/k1s0-platform/system-library-go-cache"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// 存在しないキーに対してGetを呼んだ場合にエラーなしでnilが返ることを確認する。
func TestGet_NotFound(t *testing.T) {
	c := cache.NewInMemoryCacheClient()
	val, err := c.Get(context.Background(), "missing")
	require.NoError(t, err)
	assert.Nil(t, val)
}

// SetとGetが正常に動作し、保存した値を取得できることを確認する。
func TestSetAndGet(t *testing.T) {
	c := cache.NewInMemoryCacheClient()
	ctx := context.Background()

	err := c.Set(ctx, "key1", "value1", nil)
	require.NoError(t, err)

	val, err := c.Get(ctx, "key1")
	require.NoError(t, err)
	require.NotNil(t, val)
	assert.Equal(t, "value1", *val)
}

// TTL付きでSetしたキーが期限切れ後にnilを返すことを確認する。
func TestSet_WithTTL_Expires(t *testing.T) {
	c := cache.NewInMemoryCacheClient()
	ctx := context.Background()

	ttl := 50 * time.Millisecond
	err := c.Set(ctx, "key1", "value1", &ttl)
	require.NoError(t, err)

	val, err := c.Get(ctx, "key1")
	require.NoError(t, err)
	require.NotNil(t, val)

	time.Sleep(60 * time.Millisecond)

	val, err = c.Get(ctx, "key1")
	require.NoError(t, err)
	assert.Nil(t, val)
}

// Deleteがキーを正常に削除し、2回目の削除でfalseを返すことを確認する。
func TestDelete(t *testing.T) {
	c := cache.NewInMemoryCacheClient()
	ctx := context.Background()

	_ = c.Set(ctx, "key1", "value1", nil)

	deleted, err := c.Delete(ctx, "key1")
	require.NoError(t, err)
	assert.True(t, deleted)

	deleted, err = c.Delete(ctx, "key1")
	require.NoError(t, err)
	assert.False(t, deleted)
}

// Existsがキーの存在有無を正しく返すことを確認する。
func TestExists(t *testing.T) {
	c := cache.NewInMemoryCacheClient()
	ctx := context.Background()

	exists, err := c.Exists(ctx, "key1")
	require.NoError(t, err)
	assert.False(t, exists)

	_ = c.Set(ctx, "key1", "value1", nil)

	exists, err = c.Exists(ctx, "key1")
	require.NoError(t, err)
	assert.True(t, exists)
}

// SetNXがキーが存在しない場合のみ値をセットし、既存キーには上書きしないことを確認する。
func TestSetNX(t *testing.T) {
	c := cache.NewInMemoryCacheClient()
	ctx := context.Background()
	ttl := time.Second

	ok, err := c.SetNX(ctx, "key1", "value1", ttl)
	require.NoError(t, err)
	assert.True(t, ok)

	ok, err = c.SetNX(ctx, "key1", "value2", ttl)
	require.NoError(t, err)
	assert.False(t, ok)

	val, _ := c.Get(ctx, "key1")
	assert.Equal(t, "value1", *val)
}

// SetNXがTTL期限切れのキーに対して再度セットできることを確認する。
func TestSetNX_ExpiredKey(t *testing.T) {
	c := cache.NewInMemoryCacheClient()
	ctx := context.Background()

	shortTTL := 50 * time.Millisecond
	_, _ = c.SetNX(ctx, "key1", "old", shortTTL)

	time.Sleep(60 * time.Millisecond)

	ok, err := c.SetNX(ctx, "key1", "new", time.Second)
	require.NoError(t, err)
	assert.True(t, ok)

	val, _ := c.Get(ctx, "key1")
	assert.Equal(t, "new", *val)
}

// Expireでキーに有効期限を設定し、期限切れ後にキーが消えることを確認する。
func TestExpire(t *testing.T) {
	c := cache.NewInMemoryCacheClient()
	ctx := context.Background()

	_ = c.Set(ctx, "key1", "value1", nil)

	ok, err := c.Expire(ctx, "key1", 50*time.Millisecond)
	require.NoError(t, err)
	assert.True(t, ok)

	time.Sleep(60 * time.Millisecond)

	val, _ := c.Get(ctx, "key1")
	assert.Nil(t, val)
}

// 存在しないキーにExpireを呼んだ場合にfalseが返ることを確認する。
func TestExpire_NonExistentKey(t *testing.T) {
	c := cache.NewInMemoryCacheClient()
	ctx := context.Background()

	ok, err := c.Expire(ctx, "missing", time.Second)
	require.NoError(t, err)
	assert.False(t, ok)
}
