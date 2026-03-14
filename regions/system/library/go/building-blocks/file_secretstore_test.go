package buildingblocks

import (
	"context"
	"os"
	"path/filepath"
	"testing"
)

// TestFileSecretStore_InitAndStatus は Init 前後でステータスが Uninitialized → Ready に遷移することを検証する。
func TestFileSecretStore_InitAndStatus(t *testing.T) {
	s := NewFileSecretStore(t.TempDir())
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

// TestFileSecretStore_NameVersion は Name と Version が正しい値を返すことを検証する。
func TestFileSecretStore_NameVersion(t *testing.T) {
	s := NewFileSecretStore(t.TempDir())
	if s.Name() != "file-secretstore" {
		t.Errorf("unexpected Name: %q", s.Name())
	}
	if s.Version() != "1.0.0" {
		t.Errorf("unexpected Version: %q", s.Version())
	}
}

// TestFileSecretStore_Get はファイルからシークレット値を読み込み、末尾の改行が除去されることを検証する。
func TestFileSecretStore_Get(t *testing.T) {
	dir := t.TempDir()
	// ファイルにシークレット値を書き込む（末尾改行付き）。
	if err := os.WriteFile(filepath.Join(dir, "db-password"), []byte("secret123\n"), 0600); err != nil {
		t.Fatalf("setup failed: %v", err)
	}

	s := NewFileSecretStore(dir)
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	got, err := s.Get(ctx, "db-password")
	if err != nil {
		t.Fatalf("Get failed: %v", err)
	}
	if got.Key != "db-password" {
		t.Errorf("expected Key 'db-password', got %q", got.Key)
	}
	// 末尾の改行が除去されていることを確認する。
	if got.Value != "secret123" {
		t.Errorf("expected Value 'secret123' (trimmed), got %q", got.Value)
	}
}

// TestFileSecretStore_GetTrimsCRLF は Windows 形式の CRLF 改行コードが除去されることを検証する。
func TestFileSecretStore_GetTrimsCRLF(t *testing.T) {
	dir := t.TempDir()
	if err := os.WriteFile(filepath.Join(dir, "key"), []byte("value\r\n"), 0600); err != nil {
		t.Fatalf("setup failed: %v", err)
	}

	s := NewFileSecretStore(dir)
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	got, err := s.Get(ctx, "key")
	if err != nil {
		t.Fatalf("Get failed: %v", err)
	}
	if got.Value != "value" {
		t.Errorf("expected 'value' (trimmed CRLF), got %q", got.Value)
	}
}

// TestFileSecretStore_GetNotFound は存在しないファイルを取得しようとするとエラーになることを検証する。
func TestFileSecretStore_GetNotFound(t *testing.T) {
	s := NewFileSecretStore(t.TempDir())
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	_, err := s.Get(ctx, "nonexistent")
	if err == nil {
		t.Fatal("expected error for missing file")
	}
}

// TestFileSecretStore_BulkGet は複数のファイルからシークレット値を一括取得できることを検証する。
func TestFileSecretStore_BulkGet(t *testing.T) {
	dir := t.TempDir()
	if err := os.WriteFile(filepath.Join(dir, "k1"), []byte("v1"), 0600); err != nil {
		t.Fatalf("setup failed: %v", err)
	}
	if err := os.WriteFile(filepath.Join(dir, "k2"), []byte("v2"), 0600); err != nil {
		t.Fatalf("setup failed: %v", err)
	}

	s := NewFileSecretStore(dir)
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	results, err := s.BulkGet(ctx, []string{"k1", "k2"})
	if err != nil {
		t.Fatalf("BulkGet failed: %v", err)
	}
	if len(results) != 2 {
		t.Fatalf("expected 2 results, got %d", len(results))
	}
	if results[0].Value != "v1" || results[1].Value != "v2" {
		t.Errorf("unexpected values: %v, %v", results[0].Value, results[1].Value)
	}
}

// TestFileSecretStore_BulkGetMissing は一括取得時に一つでも存在しないファイルがある場合にエラーになることを検証する。
func TestFileSecretStore_BulkGetMissing(t *testing.T) {
	dir := t.TempDir()
	if err := os.WriteFile(filepath.Join(dir, "k1"), []byte("v1"), 0600); err != nil {
		t.Fatalf("setup failed: %v", err)
	}

	s := NewFileSecretStore(dir)
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	_, err := s.BulkGet(ctx, []string{"k1", "missing"})
	if err == nil {
		t.Fatal("expected error when one file is missing")
	}
}

// TestFileSecretStore_Close は Close 後にステータスが StatusClosed に遷移することを検証する。
func TestFileSecretStore_Close(t *testing.T) {
	s := NewFileSecretStore(t.TempDir())
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	if err := s.Close(ctx); err != nil {
		t.Fatalf("Close failed: %v", err)
	}
	if s.Status(ctx) != StatusClosed {
		t.Errorf("expected StatusClosed, got %s", s.Status(ctx))
	}
}
