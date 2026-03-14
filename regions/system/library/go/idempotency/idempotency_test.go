package idempotency_test

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/k1s0-platform/system-library-go-idempotency"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// SetAndGetがべき等レコードを保存し、同じキーで取得できることを検証する。
func TestSetAndGet(t *testing.T) {
	store := idempotency.NewInMemoryIdempotencyStore()
	ctx := context.Background()

	record := idempotency.NewIdempotencyRecord("req-1", nil)
	err := store.Set(ctx, record.Key, record)
	require.NoError(t, err)

	got, err := store.Get(ctx, "req-1")
	require.NoError(t, err)
	require.NotNil(t, got)
	assert.Equal(t, "req-1", got.Key)
	assert.Equal(t, idempotency.StatusPending, got.Status)
}

// SetDuplicateが同一キーのレコードを重複登録しようとするとDUPLICATEエラーを返すことを検証する。
func TestSetDuplicate(t *testing.T) {
	store := idempotency.NewInMemoryIdempotencyStore()
	ctx := context.Background()

	record := idempotency.NewIdempotencyRecord("req-1", nil)
	require.NoError(t, store.Set(ctx, record.Key, record))

	err := store.Set(ctx, "req-1", idempotency.NewIdempotencyRecord("req-1", nil))
	require.Error(t, err)
	assert.Contains(t, err.Error(), "DUPLICATE")
}

// GetNotFoundが存在しないキーに対してnilとエラーなしを返すことを検証する。
func TestGetNotFound(t *testing.T) {
	store := idempotency.NewInMemoryIdempotencyStore()
	ctx := context.Background()

	got, err := store.Get(ctx, "missing")
	require.NoError(t, err)
	assert.Nil(t, got)
}

// MarkCompletedがレコードをCompleted状態に更新しレスポンスとステータスコードを保存することを検証する。
func TestMarkCompleted(t *testing.T) {
	store := idempotency.NewInMemoryIdempotencyStore()
	ctx := context.Background()

	record := idempotency.NewIdempotencyRecord("req-1", nil)
	require.NoError(t, store.Set(ctx, "req-1", record))

	body := []byte(`{"result":"ok"}`)
	require.NoError(t, store.MarkCompleted(ctx, "req-1", body, 200))

	got, err := store.Get(ctx, "req-1")
	require.NoError(t, err)
	require.NotNil(t, got)
	assert.Equal(t, idempotency.StatusCompleted, got.Status)
	assert.Equal(t, 200, got.StatusCode)
	assert.Equal(t, body, got.Response)
	assert.Empty(t, got.Error)
}

// MarkFailedがレコードをFailed状態に更新しエラーメッセージを保存することを検証する。
func TestMarkFailed(t *testing.T) {
	store := idempotency.NewInMemoryIdempotencyStore()
	ctx := context.Background()

	record := idempotency.NewIdempotencyRecord("req-1", nil)
	require.NoError(t, store.Set(ctx, "req-1", record))

	require.NoError(t, store.MarkFailed(ctx, "req-1", errors.New("boom")))

	got, err := store.Get(ctx, "req-1")
	require.NoError(t, err)
	require.NotNil(t, got)
	assert.Equal(t, idempotency.StatusFailed, got.Status)
	assert.Equal(t, "boom", got.Error)
	assert.Zero(t, got.StatusCode)
	assert.Nil(t, got.Response)
}

// MarkCompletedNotFoundが存在しないキーに対してNOT_FOUNDエラーを返すことを検証する。
func TestMarkCompletedNotFound(t *testing.T) {
	store := idempotency.NewInMemoryIdempotencyStore()
	ctx := context.Background()

	err := store.MarkCompleted(ctx, "missing", []byte("x"), 200)
	require.Error(t, err)
	assert.Contains(t, err.Error(), "NOT_FOUND")
}

// MarkFailedNotFoundが存在しないキーに対してNOT_FOUNDエラーを返すことを検証する。
func TestMarkFailedNotFound(t *testing.T) {
	store := idempotency.NewInMemoryIdempotencyStore()
	ctx := context.Background()

	err := store.MarkFailed(ctx, "missing", errors.New("x"))
	require.Error(t, err)
	assert.Contains(t, err.Error(), "NOT_FOUND")
}

// ExpiredRecordがTTL経過後にレコードが取得できなくなることを検証する。
func TestExpiredRecord(t *testing.T) {
	store := idempotency.NewInMemoryIdempotencyStore()
	ctx := context.Background()

	ttl := 50 * time.Millisecond
	record := idempotency.NewIdempotencyRecord("req-1", &ttl)
	require.NoError(t, store.Set(ctx, "req-1", record))

	got, err := store.Get(ctx, "req-1")
	require.NoError(t, err)
	require.NotNil(t, got)

	time.Sleep(60 * time.Millisecond)

	got, err = store.Get(ctx, "req-1")
	require.NoError(t, err)
	assert.Nil(t, got)
}

// RecordIsExpiredがIsExpiredメソッドのTTLなし・未来・過去の各ケースを正しく判定することを検証する。
func TestRecordIsExpired(t *testing.T) {
	r1 := idempotency.NewIdempotencyRecord("k1", nil)
	assert.False(t, r1.IsExpired())

	ttl := time.Hour
	r2 := idempotency.NewIdempotencyRecord("k2", &ttl)
	assert.False(t, r2.IsExpired())

	r3 := &idempotency.IdempotencyRecord{
		Key:       "k3",
		ExpiresAt: time.Now().Add(-time.Second),
	}
	assert.True(t, r3.IsExpired())
}
