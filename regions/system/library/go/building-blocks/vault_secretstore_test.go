package buildingblocks

import (
	"context"
	"errors"
	"strings"
	"testing"
)

// mockVaultClient は VaultClientIface のテスト用モック実装。
type mockVaultClient struct {
	secrets map[string]VaultSecret
	err     error
}

func (m *mockVaultClient) GetSecret(_ context.Context, path string) (VaultSecret, error) {
	if m.err != nil {
		return VaultSecret{}, m.err
	}
	vs, ok := m.secrets[path]
	if !ok {
		return VaultSecret{}, errors.New("not found")
	}
	return vs, nil
}

func TestVaultSecretStore_InitAndStatus(t *testing.T) {
	client := &mockVaultClient{secrets: map[string]VaultSecret{}}
	s := NewVaultSecretStore("vault", client)
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

func TestVaultSecretStore_NameVersion(t *testing.T) {
	client := &mockVaultClient{secrets: map[string]VaultSecret{}}
	s := NewVaultSecretStore("my-vault", client)
	if s.Name() != "my-vault" {
		t.Errorf("unexpected Name: %q", s.Name())
	}
	if s.Version() != "1.0.0" {
		t.Errorf("unexpected Version: %q", s.Version())
	}
}

func TestVaultSecretStore_GetSingleKey(t *testing.T) {
	client := &mockVaultClient{
		secrets: map[string]VaultSecret{
			"secret/db": {Path: "secret/db", Data: map[string]string{"password": "s3cr3t"}, Version: 3},
		},
	}
	s := NewVaultSecretStore("vault", client)
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	got, err := s.Get(ctx, "secret/db")
	if err != nil {
		t.Fatalf("Get failed: %v", err)
	}
	// 単一キーの場合は値をそのまま返すことを確認する。
	if got.Value != "s3cr3t" {
		t.Errorf("expected 's3cr3t', got %q", got.Value)
	}
	if got.Metadata["version"] != "3" {
		t.Errorf("expected version '3', got %q", got.Metadata["version"])
	}
}

func TestVaultSecretStore_GetMultipleKeys(t *testing.T) {
	client := &mockVaultClient{
		secrets: map[string]VaultSecret{
			"secret/cfg": {Path: "secret/cfg", Data: map[string]string{"host": "localhost", "port": "5432"}, Version: 1},
		},
	}
	s := NewVaultSecretStore("vault", client)
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	got, err := s.Get(ctx, "secret/cfg")
	if err != nil {
		t.Fatalf("Get failed: %v", err)
	}
	// 複数キーは "key=value;key=value" 形式で結合されることを確認する。
	if !strings.Contains(got.Value, "host=localhost") {
		t.Errorf("expected 'host=localhost' in value %q", got.Value)
	}
	if !strings.Contains(got.Value, "port=5432") {
		t.Errorf("expected 'port=5432' in value %q", got.Value)
	}
}

func TestVaultSecretStore_GetError(t *testing.T) {
	wantErr := errors.New("vault unavailable")
	client := &mockVaultClient{err: wantErr}
	s := NewVaultSecretStore("vault", client)
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	_, err := s.Get(ctx, "secret/any")
	if err == nil {
		t.Fatal("expected error")
	}
}

func TestVaultSecretStore_BulkGet(t *testing.T) {
	client := &mockVaultClient{
		secrets: map[string]VaultSecret{
			"secret/k1": {Data: map[string]string{"v": "val1"}, Version: 1},
			"secret/k2": {Data: map[string]string{"v": "val2"}, Version: 2},
		},
	}
	s := NewVaultSecretStore("vault", client)
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	results, err := s.BulkGet(ctx, []string{"secret/k1", "secret/k2"})
	if err != nil {
		t.Fatalf("BulkGet failed: %v", err)
	}
	if len(results) != 2 {
		t.Fatalf("expected 2 results, got %d", len(results))
	}
	if results[0].Value != "val1" || results[1].Value != "val2" {
		t.Errorf("unexpected values: %v, %v", results[0].Value, results[1].Value)
	}
}

func TestVaultSecretStore_Close(t *testing.T) {
	client := &mockVaultClient{secrets: map[string]VaultSecret{}}
	s := NewVaultSecretStore("vault", client)
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	if err := s.Close(ctx); err != nil {
		t.Fatalf("Close failed: %v", err)
	}
	if s.Status(ctx) != StatusClosed {
		t.Errorf("expected StatusClosed, got %s", s.Status(ctx))
	}
}
