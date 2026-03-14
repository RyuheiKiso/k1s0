package buildingblocks

import (
	"context"
	"io"
	"net/http"
	"net/http/httptest"
	"testing"
)

// TestHTTPOutputBinding_InitAndStatus は Init 前後でステータスが Uninitialized → Ready に遷移することを検証する。
func TestHTTPOutputBinding_InitAndStatus(t *testing.T) {
	b := NewHTTPOutputBinding(nil)
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

// TestHTTPOutputBinding_NameVersion は Name と Version が正しい値を返すことを検証する。
func TestHTTPOutputBinding_NameVersion(t *testing.T) {
	b := NewHTTPOutputBinding(nil)
	if b.Name() != "http-binding" {
		t.Errorf("unexpected Name: %q", b.Name())
	}
	if b.Version() != "1.0.0" {
		t.Errorf("unexpected Version: %q", b.Version())
	}
}

// TestHTTPOutputBinding_InvokeGET は GET リクエストを送信しレスポンスボディとステータスコードが正しく返ることを検証する。
func TestHTTPOutputBinding_InvokeGET(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodGet {
			t.Errorf("expected GET, got %s", r.Method)
		}
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte("ok"))
	}))
	defer srv.Close()

	b := NewHTTPOutputBinding(srv.Client())
	ctx := context.Background()
	_ = b.Init(ctx, Metadata{})

	resp, err := b.Invoke(ctx, http.MethodGet, nil, map[string]string{"url": srv.URL})
	if err != nil {
		t.Fatalf("Invoke failed: %v", err)
	}
	if string(resp.Data) != "ok" {
		t.Errorf("expected 'ok', got %q", resp.Data)
	}
	if resp.Metadata["status-code"] != "200" {
		t.Errorf("expected status-code '200', got %q", resp.Metadata["status-code"])
	}
}

// TestHTTPOutputBinding_InvokePOST は POST リクエストでボディと Content-Type ヘッダーが正しくサーバーへ送信されることを検証する。
func TestHTTPOutputBinding_InvokePOST(t *testing.T) {
	var receivedBody []byte
	var receivedCT string
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		receivedCT = r.Header.Get("Content-Type")
		receivedBody, _ = io.ReadAll(r.Body)
		w.WriteHeader(http.StatusCreated)
	}))
	defer srv.Close()

	b := NewHTTPOutputBinding(srv.Client())
	ctx := context.Background()
	_ = b.Init(ctx, Metadata{})

	resp, err := b.Invoke(ctx, http.MethodPost, []byte(`{"id":1}`), map[string]string{
		"url":          srv.URL,
		"content-type": "application/json",
	})
	if err != nil {
		t.Fatalf("Invoke failed: %v", err)
	}
	if resp.Metadata["status-code"] != "201" {
		t.Errorf("expected status-code '201', got %q", resp.Metadata["status-code"])
	}
	if string(receivedBody) != `{"id":1}` {
		t.Errorf("expected body '%s', got %q", `{"id":1}`, receivedBody)
	}
	if receivedCT != "application/json" {
		t.Errorf("expected Content-Type 'application/json', got %q", receivedCT)
	}
}

// TestHTTPOutputBinding_DefaultContentType は content-type 未指定でボディありのリクエストに application/octet-stream が設定されることを検証する。
func TestHTTPOutputBinding_DefaultContentType(t *testing.T) {
	var receivedCT string
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		receivedCT = r.Header.Get("Content-Type")
		w.WriteHeader(http.StatusOK)
	}))
	defer srv.Close()

	b := NewHTTPOutputBinding(srv.Client())
	ctx := context.Background()
	_ = b.Init(ctx, Metadata{})

	// content-type 未指定でデータありの場合はデフォルトの application/octet-stream が設定される。
	_, _ = b.Invoke(ctx, http.MethodPost, []byte("binary"), map[string]string{"url": srv.URL})
	if receivedCT != "application/octet-stream" {
		t.Errorf("expected 'application/octet-stream', got %q", receivedCT)
	}
}

// TestHTTPOutputBinding_CustomHeader は url と content-type 以外のメタデータがカスタムリクエストヘッダーとして転送されることを検証する。
func TestHTTPOutputBinding_CustomHeader(t *testing.T) {
	var receivedHeader string
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		receivedHeader = r.Header.Get("X-Api-Key")
		w.WriteHeader(http.StatusOK)
	}))
	defer srv.Close()

	b := NewHTTPOutputBinding(srv.Client())
	ctx := context.Background()
	_ = b.Init(ctx, Metadata{})

	// url と content-type 以外のメタデータはリクエストヘッダーとして転送される。
	_, _ = b.Invoke(ctx, http.MethodGet, nil, map[string]string{
		"url":       srv.URL,
		"X-Api-Key": "my-key",
	})
	if receivedHeader != "my-key" {
		t.Errorf("expected X-Api-Key 'my-key', got %q", receivedHeader)
	}
}

// TestHTTPOutputBinding_MissingURL はメタデータに url が含まれない場合にエラーになることを検証する。
func TestHTTPOutputBinding_MissingURL(t *testing.T) {
	b := NewHTTPOutputBinding(nil)
	ctx := context.Background()
	_ = b.Init(ctx, Metadata{})

	_, err := b.Invoke(ctx, http.MethodGet, nil, map[string]string{})
	if err == nil {
		t.Fatal("expected error when url is missing")
	}
}

// TestHTTPOutputBinding_4xxError はサーバーが 4xx を返す場合にエラーになることを検証する。
func TestHTTPOutputBinding_4xxError(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusNotFound)
		_, _ = w.Write([]byte("not found"))
	}))
	defer srv.Close()

	b := NewHTTPOutputBinding(srv.Client())
	ctx := context.Background()
	_ = b.Init(ctx, Metadata{})

	_, err := b.Invoke(ctx, http.MethodGet, nil, map[string]string{"url": srv.URL})
	if err == nil {
		t.Fatal("expected error for 4xx response")
	}
}

// TestHTTPOutputBinding_5xxError はサーバーが 5xx を返す場合にエラーになることを検証する。
func TestHTTPOutputBinding_5xxError(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusInternalServerError)
	}))
	defer srv.Close()

	b := NewHTTPOutputBinding(srv.Client())
	ctx := context.Background()
	_ = b.Init(ctx, Metadata{})

	_, err := b.Invoke(ctx, http.MethodGet, nil, map[string]string{"url": srv.URL})
	if err == nil {
		t.Fatal("expected error for 5xx response")
	}
}

// TestHTTPOutputBinding_Close は Close 後にステータスが StatusClosed に遷移することを検証する。
func TestHTTPOutputBinding_Close(t *testing.T) {
	b := NewHTTPOutputBinding(nil)
	ctx := context.Background()
	_ = b.Init(ctx, Metadata{})

	if err := b.Close(ctx); err != nil {
		t.Fatalf("Close failed: %v", err)
	}
	if b.Status(ctx) != StatusClosed {
		t.Errorf("expected StatusClosed, got %s", b.Status(ctx))
	}
}
