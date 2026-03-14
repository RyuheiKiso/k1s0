package buildingblocks

import (
	"context"
	"testing"
)

func TestEnvSecretStore_InitAndStatus(t *testing.T) {
	s := NewEnvSecretStore("")
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

func TestEnvSecretStore_NameVersion(t *testing.T) {
	s := NewEnvSecretStore("APP_")
	if s.Name() != "env-secretstore" {
		t.Errorf("unexpected Name: %q", s.Name())
	}
	if s.Version() != "1.0.0" {
		t.Errorf("unexpected Version: %q", s.Version())
	}
}

func TestEnvSecretStore_Get(t *testing.T) {
	t.Setenv("APP_DB_PASSWORD", "secret123")

	s := NewEnvSecretStore("APP_")
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	got, err := s.Get(ctx, "DB_PASSWORD")
	if err != nil {
		t.Fatalf("Get failed: %v", err)
	}
	if got.Key != "DB_PASSWORD" {
		t.Errorf("expected Key 'DB_PASSWORD', got %q", got.Key)
	}
	if got.Value != "secret123" {
		t.Errorf("expected Value 'secret123', got %q", got.Value)
	}
}

func TestEnvSecretStore_GetNoPrefix(t *testing.T) {
	t.Setenv("MY_KEY", "myvalue")

	s := NewEnvSecretStore("")
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	got, err := s.Get(ctx, "MY_KEY")
	if err != nil {
		t.Fatalf("Get failed: %v", err)
	}
	if got.Value != "myvalue" {
		t.Errorf("expected 'myvalue', got %q", got.Value)
	}
}

func TestEnvSecretStore_GetNotFound(t *testing.T) {
	s := NewEnvSecretStore("APP_")
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	_, err := s.Get(ctx, "NONEXISTENT_KEY_XYZ_12345")
	if err == nil {
		t.Fatal("expected error for missing env var")
	}
}

func TestEnvSecretStore_BulkGet(t *testing.T) {
	t.Setenv("SVC_KEY1", "val1")
	t.Setenv("SVC_KEY2", "val2")

	s := NewEnvSecretStore("SVC_")
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	results, err := s.BulkGet(ctx, []string{"KEY1", "KEY2"})
	if err != nil {
		t.Fatalf("BulkGet failed: %v", err)
	}
	if len(results) != 2 {
		t.Fatalf("expected 2 results, got %d", len(results))
	}
	if results[0].Value != "val1" || results[1].Value != "val2" {
		t.Errorf("unexpected values")
	}
}

func TestEnvSecretStore_BulkGetMissing(t *testing.T) {
	t.Setenv("SVC2_KEY1", "val1")

	s := NewEnvSecretStore("SVC2_")
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	_, err := s.BulkGet(ctx, []string{"KEY1", "MISSING_KEY_9999"})
	if err == nil {
		t.Fatal("expected error when one key is missing")
	}
}

func TestEnvSecretStore_Close(t *testing.T) {
	s := NewEnvSecretStore("")
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	if err := s.Close(ctx); err != nil {
		t.Fatalf("Close failed: %v", err)
	}
	if s.Status(ctx) != StatusClosed {
		t.Errorf("expected StatusClosed, got %s", s.Status(ctx))
	}
}
