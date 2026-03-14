// statestore_test パッケージは statestore パッケージの外部テストを提供する。
// InMemoryStateStore と RedisStateStore の各実装の動作を検証する。
package statestore_test

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/k1s0-platform/system-library-go-statestore"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// ============================================================
// InMemoryStateStore テスト
// ============================================================

// TestInMemoryStateStore_GetSet は Set 後に Get で値を取得できることを確認する。
func TestInMemoryStateStore_GetSet(t *testing.T) {
	s := statestore.NewInMemoryStateStore()
	ctx := context.Background()
	require.NoError(t, s.Init(ctx, statestore.Metadata{}))

	etag, err := s.Set(ctx, &statestore.SetRequest{Key: "k", Value: []byte("v")})
	require.NoError(t, err)
	require.NotNil(t, etag)
	assert.NotEmpty(t, etag.Value)

	entry, err := s.Get(ctx, "k")
	require.NoError(t, err)
	require.NotNil(t, entry)
	assert.Equal(t, []byte("v"), entry.Value)
	assert.Equal(t, etag.Value, entry.ETag.Value)
}

// TestInMemoryStateStore_ETagOptimisticLock は ETag 不一致の場合に ETagMismatchError が返ることを確認する。
func TestInMemoryStateStore_ETagOptimisticLock(t *testing.T) {
	s := statestore.NewInMemoryStateStore()
	ctx := context.Background()
	require.NoError(t, s.Init(ctx, statestore.Metadata{}))

	// 初回書き込みで ETag を取得する。
	_, err := s.Set(ctx, &statestore.SetRequest{Key: "k", Value: []byte("v1")})
	require.NoError(t, err)

	// 古い ETag を指定した更新は ETagMismatchError を返すことを確認する。
	_, err = s.Set(ctx, &statestore.SetRequest{Key: "k", Value: []byte("v2"), ETag: &statestore.ETag{Value: "stale"}})
	require.Error(t, err)

	var mismatch *statestore.ETagMismatchError
	assert.True(t, errors.As(err, &mismatch), "expected *ETagMismatchError, got %T", err)
}

// TestInMemoryStateStore_Delete は Delete 後に Get で nil が返ることを確認する。
func TestInMemoryStateStore_Delete(t *testing.T) {
	s := statestore.NewInMemoryStateStore()
	ctx := context.Background()
	require.NoError(t, s.Init(ctx, statestore.Metadata{}))

	etag, err := s.Set(ctx, &statestore.SetRequest{Key: "k", Value: []byte("v")})
	require.NoError(t, err)

	require.NoError(t, s.Delete(ctx, "k", etag))

	// 削除後は Get で nil が返ることを確認する。
	entry, err := s.Get(ctx, "k")
	require.NoError(t, err)
	assert.Nil(t, entry)
}

// TestInMemoryStateStore_BulkGetSet は BulkSet で複数値を設定し BulkGet で全件取得できることを確認する。
func TestInMemoryStateStore_BulkGetSet(t *testing.T) {
	s := statestore.NewInMemoryStateStore()
	ctx := context.Background()
	require.NoError(t, s.Init(ctx, statestore.Metadata{}))

	reqs := []*statestore.SetRequest{
		{Key: "a", Value: []byte("1")},
		{Key: "b", Value: []byte("2")},
	}
	etags, err := s.BulkSet(ctx, reqs)
	require.NoError(t, err)
	assert.Len(t, etags, 2)

	entries, err := s.BulkGet(ctx, []string{"a", "b"})
	require.NoError(t, err)
	require.Len(t, entries, 2)
	assert.Equal(t, []byte("1"), entries[0].Value)
	assert.Equal(t, []byte("2"), entries[1].Value)
}

// ============================================================
// RedisStateStore テスト用モック
// ============================================================

// mockCacheClient は CacheClient のテスト用モック実装。
type mockCacheClient struct {
	store   map[string]string
	err     error
	getCalls int
	setCalls int
}

// newMockCacheClient はストアマップを初期化した mockCacheClient を生成する。
func newMockCacheClient() *mockCacheClient {
	return &mockCacheClient{store: make(map[string]string)}
}

func (m *mockCacheClient) Get(_ context.Context, key string) (*string, error) {
	m.getCalls++
	if m.err != nil {
		return nil, m.err
	}
	v, ok := m.store[key]
	if !ok {
		return nil, nil
	}
	return &v, nil
}

func (m *mockCacheClient) Set(_ context.Context, key, value string, _ *time.Duration) error {
	m.setCalls++
	if m.err != nil {
		return m.err
	}
	m.store[key] = value
	return nil
}

func (m *mockCacheClient) Delete(_ context.Context, key string) (bool, error) {
	if m.err != nil {
		return false, m.err
	}
	_, ok := m.store[key]
	delete(m.store, key)
	return ok, nil
}

func (m *mockCacheClient) Exists(_ context.Context, key string) (bool, error) {
	if m.err != nil {
		return false, m.err
	}
	_, ok := m.store[key]
	return ok, nil
}

// ============================================================
// RedisStateStore テスト
// ============================================================

// TestRedisStateStore_Get はモッククライアントに対して Get が呼ばれることを確認する。
func TestRedisStateStore_Get(t *testing.T) {
	client := newMockCacheClient()
	// あらかじめ値をストアに入れておく。
	client.store["k"] = "v"
	client.store["k:__etag"] = "1"

	s := statestore.NewRedisStateStore("redis", client)
	ctx := context.Background()
	require.NoError(t, s.Init(ctx, statestore.Metadata{}))

	entry, err := s.Get(ctx, "k")
	require.NoError(t, err)
	require.NotNil(t, entry)
	assert.Equal(t, []byte("v"), entry.Value)

	// Get が呼ばれたことを確認する（値キーと ETag キーの2回）。
	assert.GreaterOrEqual(t, client.getCalls, 1)
}

// TestRedisStateStore_Set はモッククライアントに対して Set が呼ばれることを確認する。
func TestRedisStateStore_Set(t *testing.T) {
	client := newMockCacheClient()
	s := statestore.NewRedisStateStore("redis", client)
	ctx := context.Background()
	require.NoError(t, s.Init(ctx, statestore.Metadata{}))

	etag, err := s.Set(ctx, &statestore.SetRequest{Key: "k", Value: []byte("v")})
	require.NoError(t, err)
	require.NotNil(t, etag)
	assert.NotEmpty(t, etag.Value)

	// Set が呼ばれたことを確認する（値と ETag の2回）。
	assert.Equal(t, 2, client.setCalls)
}

// TestRedisStateStore_ETagMismatch は ETag 不一致の場合にエラーが返ることを確認する。
func TestRedisStateStore_ETagMismatch(t *testing.T) {
	client := newMockCacheClient()
	s := statestore.NewRedisStateStore("redis", client)
	ctx := context.Background()
	require.NoError(t, s.Init(ctx, statestore.Metadata{}))

	// 初回書き込みで ETag を確立する。
	_, err := s.Set(ctx, &statestore.SetRequest{Key: "k", Value: []byte("v1")})
	require.NoError(t, err)

	// 古い ETag を指定した更新は ETagMismatchError を返すことを確認する。
	_, err = s.Set(ctx, &statestore.SetRequest{Key: "k", Value: []byte("v2"), ETag: &statestore.ETag{Value: "stale"}})
	require.Error(t, err)

	var mismatch *statestore.ETagMismatchError
	assert.True(t, errors.As(err, &mismatch), "expected *ETagMismatchError, got %T", err)
}
