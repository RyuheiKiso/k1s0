package buildingblocks

import (
	"context"
	"testing"
)

func TestInMemorySecretStore_InitAndStatus(t *testing.T) {
	s := NewInMemorySecretStore()
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

func TestInMemorySecretStore_Name(t *testing.T) {
	s := NewInMemorySecretStore()
	if s.Name() != "inmemory-secretstore" {
		t.Errorf("unexpected Name: %q", s.Name())
	}
	if s.Version() != "1.0.0" {
		t.Errorf("unexpected Version: %q", s.Version())
	}
}

func TestInMemorySecretStore_GetPut(t *testing.T) {
	s := NewInMemorySecretStore()
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	s.Put("db-password", "secret123")

	got, err := s.Get(ctx, "db-password")
	if err != nil {
		t.Fatalf("Get failed: %v", err)
	}
	if got.Key != "db-password" {
		t.Errorf("expected Key 'db-password', got %q", got.Key)
	}
	if got.Value != "secret123" {
		t.Errorf("expected Value 'secret123', got %q", got.Value)
	}
}

func TestInMemorySecretStore_GetNotFound(t *testing.T) {
	s := NewInMemorySecretStore()
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	_, err := s.Get(ctx, "missing")
	if err == nil {
		t.Fatal("expected error for missing key")
	}
}

func TestInMemorySecretStore_BulkGet(t *testing.T) {
	s := NewInMemorySecretStore()
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	s.Put("k1", "v1")
	s.Put("k2", "v2")

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

func TestInMemorySecretStore_BulkGetMissing(t *testing.T) {
	s := NewInMemorySecretStore()
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	s.Put("k1", "v1")
	_, err := s.BulkGet(ctx, []string{"k1", "missing"})
	if err == nil {
		t.Fatal("expected error for missing key in BulkGet")
	}
}

func TestInMemorySecretStore_Close(t *testing.T) {
	s := NewInMemorySecretStore()
	ctx := context.Background()
	_ = s.Init(ctx, Metadata{})

	if err := s.Close(ctx); err != nil {
		t.Fatalf("Close failed: %v", err)
	}
	if s.Status(ctx) != StatusClosed {
		t.Errorf("expected StatusClosed, got %s", s.Status(ctx))
	}
}
