package idempotency_test

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/alicebob/miniredis/v2"
	"github.com/k1s0-platform/system-library-go-idempotency"
	"github.com/redis/go-redis/v9"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// RedisStoreSetGetAndDuplicateがRedisストアでのレコード登録・取得・重複エラーを正しく処理することを検証する。
func TestRedisStoreSetGetAndDuplicate(t *testing.T) {
	srv, err := miniredis.Run()
	require.NoError(t, err)
	defer srv.Close()

	client := redis.NewClient(&redis.Options{Addr: srv.Addr()})
	store := idempotency.NewRedisIdempotencyStore(client, idempotency.WithRedisDefaultTTL(time.Hour))
	ctx := context.Background()

	record := idempotency.NewIdempotencyRecord("req-1", nil)
	require.NoError(t, store.Set(ctx, "req-1", record))

	got, err := store.Get(ctx, "req-1")
	require.NoError(t, err)
	require.NotNil(t, got)
	assert.Equal(t, idempotency.StatusPending, got.Status)

	err = store.Set(ctx, "req-1", idempotency.NewIdempotencyRecord("req-1", nil))
	require.Error(t, err)
	assert.Contains(t, err.Error(), "DUPLICATE")
}

// RedisStoreMarkCompletedAndFailedがRedisストアでCompleted・Failed状態への遷移を正しく処理することを検証する。
func TestRedisStoreMarkCompletedAndFailed(t *testing.T) {
	srv, err := miniredis.Run()
	require.NoError(t, err)
	defer srv.Close()

	client := redis.NewClient(&redis.Options{Addr: srv.Addr()})
	store := idempotency.NewRedisIdempotencyStore(client, idempotency.WithRedisDefaultTTL(time.Hour))
	ctx := context.Background()

	require.NoError(t, store.Set(ctx, "req-1", idempotency.NewIdempotencyRecord("req-1", nil)))
	require.NoError(t, store.MarkCompleted(ctx, "req-1", []byte(`{"ok":true}`), 200))

	got, err := store.Get(ctx, "req-1")
	require.NoError(t, err)
	require.NotNil(t, got)
	assert.Equal(t, idempotency.StatusCompleted, got.Status)
	assert.Equal(t, 200, got.StatusCode)

	require.NoError(t, store.MarkFailed(ctx, "req-1", errors.New("backend unavailable")))
	got, err = store.Get(ctx, "req-1")
	require.NoError(t, err)
	require.NotNil(t, got)
	assert.Equal(t, idempotency.StatusFailed, got.Status)
	assert.Equal(t, "backend unavailable", got.Error)
}
