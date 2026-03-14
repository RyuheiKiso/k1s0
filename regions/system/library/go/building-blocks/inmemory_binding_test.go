package buildingblocks

import (
	"context"
	"errors"
	"testing"
)

// InMemoryOutputBinding の Init 前後でステータスが Uninitialized → Ready に遷移することを確認する。
func TestInMemoryOutputBinding_InitAndStatus(t *testing.T) {
	b := NewInMemoryOutputBinding()
	ctx := context.Background()

	if b.Status(ctx) != StatusUninitialized {
		t.Errorf("expected StatusUninitialized, got %s", b.Status(ctx))
	}
	if err := b.Init(ctx, Metadata{}); err != nil {
		t.Fatalf("Init failed: %v", err)
	}
	if b.Status(ctx) != StatusReady {
		t.Errorf("expected StatusReady, got %s", b.Status(ctx))
	}
}

// InMemoryOutputBinding の Name と Version が正しい値を返すことを確認する。
func TestInMemoryOutputBinding_Name(t *testing.T) {
	b := NewInMemoryOutputBinding()
	if b.Name() != "inmemory-binding" {
		t.Errorf("unexpected Name: %q", b.Name())
	}
	if b.Version() != "1.0.0" {
		t.Errorf("unexpected Version: %q", b.Version())
	}
}

// InMemoryOutputBinding の Invoke が呼び出し内容を LastInvocation に記録することを確認する。
func TestInMemoryOutputBinding_InvokeRecords(t *testing.T) {
	b := NewInMemoryOutputBinding()
	ctx := context.Background()
	_ = b.Init(ctx, Metadata{})

	if b.LastInvocation() != nil {
		t.Error("expected nil LastInvocation before any Invoke")
	}

	resp, err := b.Invoke(ctx, "send", []byte("payload"), map[string]string{"key": "val"})
	if err != nil {
		t.Fatalf("Invoke failed: %v", err)
	}
	if resp == nil {
		t.Fatal("expected non-nil response")
	}

	inv := b.LastInvocation()
	if inv == nil {
		t.Fatal("expected LastInvocation to be set")
	}
	if inv.Operation != "send" {
		t.Errorf("expected Operation 'send', got %q", inv.Operation)
	}
	if string(inv.Data) != "payload" {
		t.Errorf("expected Data 'payload', got %q", inv.Data)
	}
	if inv.Metadata["key"] != "val" {
		t.Errorf("expected Metadata key 'val', got %q", inv.Metadata["key"])
	}
}

// InMemoryOutputBinding の SetResponse で事前設定したレスポンスが Invoke から返ることを確認する。
func TestInMemoryOutputBinding_SetResponse(t *testing.T) {
	b := NewInMemoryOutputBinding()
	ctx := context.Background()
	_ = b.Init(ctx, Metadata{})

	b.SetResponse(&BindingResponse{Data: []byte("ok")}, nil)

	resp, err := b.Invoke(ctx, "op", nil, nil)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if string(resp.Data) != "ok" {
		t.Errorf("expected 'ok', got %q", resp.Data)
	}
}

// InMemoryOutputBinding の SetResponse でエラーを設定すると Invoke がそのエラーを返すことを確認する。
func TestInMemoryOutputBinding_SetResponseError(t *testing.T) {
	b := NewInMemoryOutputBinding()
	ctx := context.Background()
	_ = b.Init(ctx, Metadata{})

	want := errors.New("invoke error")
	b.SetResponse(nil, want)

	_, err := b.Invoke(ctx, "op", nil, nil)
	if !errors.Is(err, want) {
		t.Errorf("expected %v, got %v", want, err)
	}
}

// InMemoryOutputBinding の Reset が LastInvocation とレスポンスをクリアすることを確認する。
func TestInMemoryOutputBinding_Reset(t *testing.T) {
	b := NewInMemoryOutputBinding()
	ctx := context.Background()
	_ = b.Init(ctx, Metadata{})

	b.SetResponse(&BindingResponse{Data: []byte("x")}, nil)
	_, _ = b.Invoke(ctx, "op", nil, nil)

	b.Reset()

	if b.LastInvocation() != nil {
		t.Error("expected nil LastInvocation after Reset")
	}
	// After reset, default empty response is returned.
	resp, err := b.Invoke(ctx, "op2", nil, nil)
	if err != nil {
		t.Fatalf("unexpected error after Reset: %v", err)
	}
	if resp.Data != nil {
		t.Errorf("expected nil Data after Reset, got %v", resp.Data)
	}
}

// InMemoryOutputBinding の Close がステータスを StatusClosed に遷移させることを確認する。
func TestInMemoryOutputBinding_Close(t *testing.T) {
	b := NewInMemoryOutputBinding()
	ctx := context.Background()
	_ = b.Init(ctx, Metadata{})

	if err := b.Close(ctx); err != nil {
		t.Fatalf("Close failed: %v", err)
	}
	if b.Status(ctx) != StatusClosed {
		t.Errorf("expected StatusClosed, got %s", b.Status(ctx))
	}
}
