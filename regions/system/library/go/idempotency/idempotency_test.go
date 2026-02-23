package idempotency_test

import (
	"context"
	"testing"
	"time"

	"github.com/k1s0-platform/system-library-go-idempotency"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestInsertAndGet(t *testing.T) {
	store := idempotency.NewInMemoryIdempotencyStore()
	ctx := context.Background()

	record := idempotency.NewIdempotencyRecord("req-1", nil)
	err := store.Insert(ctx, record)
	require.NoError(t, err)

	got, err := store.Get(ctx, "req-1")
	require.NoError(t, err)
	require.NotNil(t, got)
	assert.Equal(t, "req-1", got.Key)
	assert.Equal(t, idempotency.StatusPending, got.Status)
}

func TestInsert_Duplicate(t *testing.T) {
	store := idempotency.NewInMemoryIdempotencyStore()
	ctx := context.Background()

	record := idempotency.NewIdempotencyRecord("req-1", nil)
	_ = store.Insert(ctx, record)

	err := store.Insert(ctx, idempotency.NewIdempotencyRecord("req-1", nil))
	require.Error(t, err)
	assert.Contains(t, err.Error(), "DUPLICATE")
}

func TestGet_NotFound(t *testing.T) {
	store := idempotency.NewInMemoryIdempotencyStore()
	ctx := context.Background()

	got, err := store.Get(ctx, "missing")
	require.NoError(t, err)
	assert.Nil(t, got)
}

func TestUpdate(t *testing.T) {
	store := idempotency.NewInMemoryIdempotencyStore()
	ctx := context.Background()

	record := idempotency.NewIdempotencyRecord("req-1", nil)
	_ = store.Insert(ctx, record)

	body := `{"result": "ok"}`
	status := 200
	err := store.Update(ctx, "req-1", idempotency.StatusCompleted, &body, &status)
	require.NoError(t, err)

	got, _ := store.Get(ctx, "req-1")
	assert.Equal(t, idempotency.StatusCompleted, got.Status)
	assert.Equal(t, &body, got.ResponseBody)
	assert.Equal(t, &status, got.ResponseStatus)
	assert.NotNil(t, got.CompletedAt)
}

func TestUpdate_NotFound(t *testing.T) {
	store := idempotency.NewInMemoryIdempotencyStore()
	ctx := context.Background()

	err := store.Update(ctx, "missing", idempotency.StatusCompleted, nil, nil)
	require.Error(t, err)
	assert.Contains(t, err.Error(), "NOT_FOUND")
}

func TestDelete(t *testing.T) {
	store := idempotency.NewInMemoryIdempotencyStore()
	ctx := context.Background()

	_ = store.Insert(ctx, idempotency.NewIdempotencyRecord("req-1", nil))

	deleted, err := store.Delete(ctx, "req-1")
	require.NoError(t, err)
	assert.True(t, deleted)

	deleted, err = store.Delete(ctx, "req-1")
	require.NoError(t, err)
	assert.False(t, deleted)
}

func TestExpiredRecord(t *testing.T) {
	store := idempotency.NewInMemoryIdempotencyStore()
	ctx := context.Background()

	ttl := 50 * time.Millisecond
	record := idempotency.NewIdempotencyRecord("req-1", &ttl)
	_ = store.Insert(ctx, record)

	got, _ := store.Get(ctx, "req-1")
	require.NotNil(t, got)

	time.Sleep(60 * time.Millisecond)

	got, err := store.Get(ctx, "req-1")
	require.NoError(t, err)
	assert.Nil(t, got)
}

func TestRecord_IsExpired(t *testing.T) {
	// 期限なし
	r1 := idempotency.NewIdempotencyRecord("k1", nil)
	assert.False(t, r1.IsExpired())

	// 期限あり（未来）
	ttl := time.Hour
	r2 := idempotency.NewIdempotencyRecord("k2", &ttl)
	assert.False(t, r2.IsExpired())

	// 期限あり（過去）
	past := time.Now().Add(-time.Second)
	r3 := &idempotency.IdempotencyRecord{Key: "k3", ExpiresAt: &past}
	assert.True(t, r3.IsExpired())
}
