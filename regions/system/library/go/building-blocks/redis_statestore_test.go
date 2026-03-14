package buildingblocks

import (
	"context"
	"errors"
	"testing"
	"time"
)

// mockCacheClient は CacheClient のテスト用モック実装。
type mockCacheClient struct {
	store map[string]string
	err   error
}

// newMockCacheClient はストアマップを初期化した mockCacheClient を生成する。
func newMockCacheClient() *mockCacheClient {
	return &mockCacheClient{store: make(map[string]string)}
}

func (m *mockCacheClient) Get(_ context.Context, key string) (*string, error) {
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

// TestRedisStateStore_InitAndStatus は Init 前後でステータスが Uninitialized → Ready に遷移することを検証する。
func TestRedisStateStore_InitAndStatus(t *testing.T) {
	s := NewRedisStateStore("redis", newMockCacheClient())
	ctx := context.Background()

	if s.Status(ctx) != StatusUninitialized {
		t.Errorf("expected StatusUninitialized, got %s", s.Status(ctx))
	}
	if err := s.Init(ctx, Metadata{}); err != nil {
		t.Fatalf("Init failed: %v", err)
	}
	if s.Status(ctx) != StatusReady {
		t.Errorf("expected StatusReady, got %s", s.Status(ctx))
	}
}

// TestRedisStateStore_NameVersion は Name と Version が正しい値を返すことを検証する。
func TestRedisStateStore_NameVersion(t *testing.T) {
	s := NewRedisStateStore("my-redis", newMockCacheClient())
	if s.Name() != "my-redis" {
		t.Errorf("unexpected Name: %q", s.Name())
	}
	if s.Version() != "1.0.0" {
		t.Errorf("unexpected Version: %q", s.Version())
	}
}

// TestRedisStateStore_SetGet は値を Set した後に Get で同じ値と ETag を取得できることを検証する。
func TestRedisStateStore_SetGet(t *testing.T) {
	s := NewRedisStateStore("redis", newMockCacheClient())
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	etag, err := s.Set(ctx, &SetRequest{Key: "k", Value: []byte("v")})
	if err != nil {
		t.Fatalf("Set failed: %v", err)
	}
	if etag == nil || etag.Value == "" {
		t.Fatal("expected non-empty ETag")
	}

	entry, err := s.Get(ctx, "k")
	if err != nil {
		t.Fatalf("Get failed: %v", err)
	}
	if entry == nil {
		t.Fatal("expected entry, got nil")
	}
	if string(entry.Value) != "v" {
		t.Errorf("expected Value 'v', got %q", entry.Value)
	}
	if entry.ETag.Value != etag.Value {
		t.Errorf("ETag mismatch: got %q, want %q", entry.ETag.Value, etag.Value)
	}
}

// TestRedisStateStore_GetMissing は存在しないキーを Get するとエラーなしで nil が返ることを検証する。
func TestRedisStateStore_GetMissing(t *testing.T) {
	s := NewRedisStateStore("redis", newMockCacheClient())
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	entry, err := s.Get(ctx, "missing")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if entry != nil {
		t.Error("expected nil for missing key")
	}
}

// TestRedisStateStore_SetWithETag は正しい ETag を指定して Set すると新しい ETag が発行されることを検証する。
func TestRedisStateStore_SetWithETag(t *testing.T) {
	s := NewRedisStateStore("redis", newMockCacheClient())
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	etag, _ := s.Set(ctx, &SetRequest{Key: "k", Value: []byte("v1")})

	etag2, err := s.Set(ctx, &SetRequest{Key: "k", Value: []byte("v2"), ETag: etag})
	if err != nil {
		t.Fatalf("Set with correct ETag failed: %v", err)
	}
	if etag2.Value == etag.Value {
		t.Error("expected new ETag after update")
	}
}

// TestRedisStateStore_SetETagMismatch は古い ETag で Set すると ETagMismatchError が返ることを検証する。
func TestRedisStateStore_SetETagMismatch(t *testing.T) {
	s := NewRedisStateStore("redis", newMockCacheClient())
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	_, _ = s.Set(ctx, &SetRequest{Key: "k", Value: []byte("v1")})

	_, err := s.Set(ctx, &SetRequest{Key: "k", Value: []byte("v2"), ETag: &ETag{Value: "stale"}})
	if err == nil {
		t.Fatal("expected ETagMismatchError")
	}
	var mismatch *ETagMismatchError
	if !errors.As(err, &mismatch) {
		t.Errorf("expected *ETagMismatchError, got %T", err)
	}
}

// TestRedisStateStore_SetETagOnMissingKey は存在しないキーに ETag 付きで Set すると ETagMismatchError が返ることを検証する。
func TestRedisStateStore_SetETagOnMissingKey(t *testing.T) {
	s := NewRedisStateStore("redis", newMockCacheClient())
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	_, err := s.Set(ctx, &SetRequest{Key: "k", Value: []byte("v"), ETag: &ETag{Value: "1"}})
	if err == nil {
		t.Fatal("expected ETagMismatchError when key does not exist")
	}
	var mismatch *ETagMismatchError
	if !errors.As(err, &mismatch) {
		t.Errorf("expected *ETagMismatchError, got %T", err)
	}
}

// TestRedisStateStore_Delete は正しい ETag を指定して Delete するとキーが削除されることを検証する。
func TestRedisStateStore_Delete(t *testing.T) {
	s := NewRedisStateStore("redis", newMockCacheClient())
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	etag, _ := s.Set(ctx, &SetRequest{Key: "k", Value: []byte("v")})

	if err := s.Delete(ctx, "k", etag); err != nil {
		t.Fatalf("Delete failed: %v", err)
	}

	entry, _ := s.Get(ctx, "k")
	if entry != nil {
		t.Error("expected nil after deletion")
	}
}

// TestRedisStateStore_DeleteETagMismatch は古い ETag で Delete すると ETagMismatchError が返ることを検証する。
func TestRedisStateStore_DeleteETagMismatch(t *testing.T) {
	s := NewRedisStateStore("redis", newMockCacheClient())
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	_, _ = s.Set(ctx, &SetRequest{Key: "k", Value: []byte("v")})

	err := s.Delete(ctx, "k", &ETag{Value: "stale"})
	if err == nil {
		t.Fatal("expected ETagMismatchError")
	}
	var mismatch *ETagMismatchError
	if !errors.As(err, &mismatch) {
		t.Errorf("expected *ETagMismatchError, got %T", err)
	}
}

// TestRedisStateStore_DeleteMissingKey は ETag なしで存在しないキーを Delete してもエラーが発生しないことを検証する。
func TestRedisStateStore_DeleteMissingKey(t *testing.T) {
	s := NewRedisStateStore("redis", newMockCacheClient())
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	// ETag なしで存在しないキーを削除してもエラーにならないことを確認する。
	if err := s.Delete(ctx, "missing", nil); err != nil {
		t.Errorf("expected no error deleting missing key, got: %v", err)
	}
}

// TestRedisStateStore_BulkSetGet は複数の値を BulkSet した後に BulkGet で全て取得できることを検証する。
func TestRedisStateStore_BulkSetGet(t *testing.T) {
	s := NewRedisStateStore("redis", newMockCacheClient())
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	reqs := []*SetRequest{
		{Key: "a", Value: []byte("1")},
		{Key: "b", Value: []byte("2")},
	}
	etags, err := s.BulkSet(ctx, reqs)
	if err != nil {
		t.Fatalf("BulkSet failed: %v", err)
	}
	if len(etags) != 2 {
		t.Fatalf("expected 2 ETags, got %d", len(etags))
	}

	entries, err := s.BulkGet(ctx, []string{"a", "b"})
	if err != nil {
		t.Fatalf("BulkGet failed: %v", err)
	}
	if len(entries) != 2 {
		t.Fatalf("expected 2 entries, got %d", len(entries))
	}
	if string(entries[0].Value) != "1" || string(entries[1].Value) != "2" {
		t.Errorf("unexpected values")
	}
}

// TestRedisStateStore_GetError は Redis クライアントがエラーを返す場合に Get がエラーになることを検証する。
func TestRedisStateStore_GetError(t *testing.T) {
	client := newMockCacheClient()
	client.err = errors.New("redis error")
	s := NewRedisStateStore("redis", client)
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	_, err := s.Get(ctx, "k")
	if err == nil {
		t.Fatal("expected error from Redis client")
	}
}

// TestRedisStateStore_Close は Close 後にステータスが StatusClosed に遷移することを検証する。
func TestRedisStateStore_Close(t *testing.T) {
	s := NewRedisStateStore("redis", newMockCacheClient())
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	if err := s.Close(ctx); err != nil {
		t.Fatalf("Close failed: %v", err)
	}
	if s.Status(ctx) != StatusClosed {
		t.Errorf("expected StatusClosed, got %s", s.Status(ctx))
	}
}
