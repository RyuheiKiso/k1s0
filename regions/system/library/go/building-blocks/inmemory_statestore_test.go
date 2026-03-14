package buildingblocks

import (
	"context"
	"errors"
	"testing"
)

// InMemoryStateStore の Init 前後でステータスが Uninitialized → Ready に遷移することを確認する。
func TestInMemoryStateStore_InitAndStatus(t *testing.T) {
	s := NewInMemoryStateStore()
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

// InMemoryStateStore の Name と Version が正しい値を返すことを確認する。
func TestInMemoryStateStore_Name(t *testing.T) {
	s := NewInMemoryStateStore()
	if s.Name() != "inmemory-statestore" {
		t.Errorf("unexpected Name: %q", s.Name())
	}
	if s.Version() != "1.0.0" {
		t.Errorf("unexpected Version: %q", s.Version())
	}
}

// InMemoryStateStore に Set した値を Get で正しく取得でき ETag が発行されることを確認する。
func TestInMemoryStateStore_SetGet(t *testing.T) {
	s := NewInMemoryStateStore()
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

// InMemoryStateStore が存在しないキーを Get するとエラーなしで nil を返すことを確認する。
func TestInMemoryStateStore_GetMissing(t *testing.T) {
	s := NewInMemoryStateStore()
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

// InMemoryStateStore が正しい ETag を指定した Set で更新に成功し新しい ETag を返すことを確認する。
func TestInMemoryStateStore_SetWithETag(t *testing.T) {
	s := NewInMemoryStateStore()
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	etag, _ := s.Set(ctx, &SetRequest{Key: "k", Value: []byte("v1")})

	// Update with correct ETag succeeds.
	etag2, err := s.Set(ctx, &SetRequest{Key: "k", Value: []byte("v2"), ETag: etag})
	if err != nil {
		t.Fatalf("Set with correct ETag failed: %v", err)
	}
	if etag2.Value == etag.Value {
		t.Error("expected new ETag after update")
	}
}

// InMemoryStateStore が古い ETag を指定した Set で ETagMismatchError を返すことを確認する。
func TestInMemoryStateStore_SetETagMismatch(t *testing.T) {
	s := NewInMemoryStateStore()
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	_, _ = s.Set(ctx, &SetRequest{Key: "k", Value: []byte("v1")})

	// Update with stale ETag fails.
	_, err := s.Set(ctx, &SetRequest{Key: "k", Value: []byte("v2"), ETag: &ETag{Value: "stale"}})
	if err == nil {
		t.Fatal("expected ETagMismatchError")
	}
	var mismatch *ETagMismatchError
	if !errors.As(err, &mismatch) {
		t.Errorf("expected *ETagMismatchError, got %T", err)
	}
}

// InMemoryStateStore が存在しないキーに ETag 付きで Set すると ETagMismatchError を返すことを確認する。
func TestInMemoryStateStore_SetETagOnMissingKey(t *testing.T) {
	s := NewInMemoryStateStore()
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	_, err := s.Set(ctx, &SetRequest{Key: "k", Value: []byte("v"), ETag: &ETag{Value: "1"}})
	if err == nil {
		t.Fatal("expected ETagMismatchError when key does not exist")
	}
}

// InMemoryStateStore が正しい ETag を指定した Delete でキーを削除することを確認する。
func TestInMemoryStateStore_Delete(t *testing.T) {
	s := NewInMemoryStateStore()
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

// InMemoryStateStore が古い ETag を指定した Delete で ETagMismatchError を返すことを確認する。
func TestInMemoryStateStore_DeleteETagMismatch(t *testing.T) {
	s := NewInMemoryStateStore()
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	_, _ = s.Set(ctx, &SetRequest{Key: "k", Value: []byte("v")})

	err := s.Delete(ctx, "k", &ETag{Value: "stale"})
	if err == nil {
		t.Fatal("expected ETagMismatchError")
	}
}

// InMemoryStateStore が存在しないキーを ETag なしで Delete してもエラーにならないことを確認する。
func TestInMemoryStateStore_DeleteMissingKey(t *testing.T) {
	s := NewInMemoryStateStore()
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	if err := s.Delete(ctx, "missing", nil); err != nil {
		t.Errorf("expected no error deleting missing key, got: %v", err)
	}
}

// InMemoryStateStore の BulkSet で複数の値を設定し BulkGet で全件取得できることを確認する。
func TestInMemoryStateStore_BulkSetGet(t *testing.T) {
	s := NewInMemoryStateStore()
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

// InMemoryStateStore の Close がステータスを StatusClosed に遷移させることを確認する。
func TestInMemoryStateStore_Close(t *testing.T) {
	s := NewInMemoryStateStore()
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	if err := s.Close(ctx); err != nil {
		t.Fatalf("Close failed: %v", err)
	}
	if s.Status(ctx) != StatusClosed {
		t.Errorf("expected StatusClosed, got %s", s.Status(ctx))
	}
}
