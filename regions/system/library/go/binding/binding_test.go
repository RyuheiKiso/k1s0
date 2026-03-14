// binding_test パッケージは binding パッケージの各実装に対するユニットテストを提供する。
// InMemoryOutputBinding、HTTPOutputBinding、FileOutputBinding の動作を検証する。
package binding_test

import (
	"context"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	binding "github.com/k1s0-platform/system-library-go-binding"
)

// ─────────────────────────────────────────
// InMemoryOutputBinding テスト
// ─────────────────────────────────────────

// TestInMemoryOutputBinding_Invoke は Invoke を呼んで LastInvocation で呼び出し記録を確認する。
func TestInMemoryOutputBinding_Invoke(t *testing.T) {
	ctx := context.Background()
	b := binding.NewInMemoryOutputBinding()

	// Invoke を呼び出して記録されることを検証する。
	resp, err := b.Invoke(ctx, "send", []byte("hello"), map[string]string{"key": "val"})
	require.NoError(t, err)
	assert.NotNil(t, resp)

	// LastInvocation で呼び出し情報が記録されていることを確認する。
	last := b.LastInvocation()
	require.NotNil(t, last)
	assert.Equal(t, "send", last.Operation)
	assert.Equal(t, []byte("hello"), last.Data)
	assert.Equal(t, "val", last.Metadata["key"])
}

// TestInMemoryOutputBinding_SetResponse は SetResponse で設定したレスポンスが Invoke から返ることを確認する。
func TestInMemoryOutputBinding_SetResponse(t *testing.T) {
	ctx := context.Background()
	b := binding.NewInMemoryOutputBinding()

	// モックレスポンスを設定してから Invoke を呼ぶ。
	expected := &binding.BindingResponse{Data: []byte("mock-data"), Metadata: map[string]string{"status": "ok"}}
	b.SetResponse(expected, nil)

	resp, err := b.Invoke(ctx, "op", nil, nil)
	require.NoError(t, err)
	assert.Equal(t, expected.Data, resp.Data)
	assert.Equal(t, "ok", resp.Metadata["status"])
}

// TestInMemoryOutputBinding_Reset は Reset 後に LastInvocation が nil になることを確認する。
func TestInMemoryOutputBinding_Reset(t *testing.T) {
	ctx := context.Background()
	b := binding.NewInMemoryOutputBinding()

	// 一度 Invoke して記録を作ってから Reset を呼ぶ。
	_, err := b.Invoke(ctx, "op", nil, nil)
	require.NoError(t, err)
	require.NotNil(t, b.LastInvocation())

	// Reset 後は呼び出し記録が消えていることを確認する。
	b.Reset()
	assert.Nil(t, b.LastInvocation())
}

// ─────────────────────────────────────────
// HTTPOutputBinding テスト
// ─────────────────────────────────────────

// TestHTTPOutputBinding_NoURL は metadata["url"] なしで Invoke を呼ぶとエラーが返ることを確認する。
func TestHTTPOutputBinding_NoURL(t *testing.T) {
	ctx := context.Background()
	b := binding.NewHTTPOutputBinding(nil)

	// URL を指定しない場合はエラーが返ることを検証する。
	_, err := b.Invoke(ctx, "GET", nil, map[string]string{})
	require.Error(t, err)
	assert.Contains(t, err.Error(), `metadata["url"] is required`)
}

// TestHTTPOutputBinding_GET は httptest.NewServer を使って GET リクエストが成功することを確認する。
func TestHTTPOutputBinding_GET(t *testing.T) {
	// テスト用 HTTP サーバーを起動して GET リクエストを受け付ける。
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, http.MethodGet, r.Method)
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte("pong"))
	}))
	defer srv.Close()

	ctx := context.Background()
	b := binding.NewHTTPOutputBinding(srv.Client())

	// テストサーバーに GET リクエストを送信して成功することを検証する。
	resp, err := b.Invoke(ctx, "GET", nil, map[string]string{"url": srv.URL})
	require.NoError(t, err)
	assert.Equal(t, "pong", string(resp.Data))
	assert.Equal(t, "200", resp.Metadata["status-code"])
}

// TestHTTPOutputBinding_4xxError は 400 レスポンスで Invoke がエラーを返すことを確認する。
func TestHTTPOutputBinding_4xxError(t *testing.T) {
	// 400 を返すテスト用サーバーを起動する。
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		http.Error(w, "bad request", http.StatusBadRequest)
	}))
	defer srv.Close()

	ctx := context.Background()
	b := binding.NewHTTPOutputBinding(srv.Client())

	// 4xx レスポンスはエラーとして扱われることを検証する。
	_, err := b.Invoke(ctx, "GET", nil, map[string]string{"url": srv.URL})
	require.Error(t, err)
	assert.Contains(t, err.Error(), "HTTP 400")
}

// ─────────────────────────────────────────
// FileOutputBinding テスト
// ─────────────────────────────────────────

// mockFileClient はテスト用の FileClientIface モック実装。
type mockFileClient struct {
	// uploadURLCalled は GenerateUploadURL が呼ばれた回数。
	uploadURLCalled int
	// uploadPath は GenerateUploadURL に渡されたパス。
	uploadPath string
}

// GenerateUploadURL はテスト用のモック実装で呼び出しを記録する。
func (m *mockFileClient) GenerateUploadURL(_ context.Context, path, _ string, _ time.Duration) (*binding.FilePresignedURL, error) {
	m.uploadURLCalled++
	m.uploadPath = path
	return &binding.FilePresignedURL{
		URL:    "https://example.com/upload",
		Method: "PUT",
	}, nil
}

// GenerateDownloadURL はテスト用のモック実装（未使用）。
func (m *mockFileClient) GenerateDownloadURL(_ context.Context, _ string, _ time.Duration) (*binding.FilePresignedURL, error) {
	return &binding.FilePresignedURL{URL: "https://example.com/download", Method: "GET"}, nil
}

// Delete はテスト用のモック実装（未使用）。
func (m *mockFileClient) Delete(_ context.Context, _ string) error { return nil }

// List はテスト用のモック実装（未使用）。
func (m *mockFileClient) List(_ context.Context, _ string) ([]*binding.FileInfo, error) {
	return []*binding.FileInfo{}, nil
}

// Copy はテスト用のモック実装（未使用）。
func (m *mockFileClient) Copy(_ context.Context, _, _ string) error { return nil }

// TestFileOutputBinding_UnsupportedOperation は未知の operation でエラーが返ることを確認する。
func TestFileOutputBinding_UnsupportedOperation(t *testing.T) {
	ctx := context.Background()
	client := &mockFileClient{}
	b := binding.NewFileOutputBinding("test-file-binding", client)

	// サポートされていない operation を指定した場合はエラーが返ることを検証する。
	_, err := b.Invoke(ctx, "unknown-op", nil, map[string]string{})
	require.Error(t, err)
	assert.Contains(t, err.Error(), "unsupported operation")
}

// TestFileOutputBinding_UploadURL はモッククライアントで upload-url が正しく呼ばれることを確認する。
func TestFileOutputBinding_UploadURL(t *testing.T) {
	ctx := context.Background()
	client := &mockFileClient{}
	b := binding.NewFileOutputBinding("test-file-binding", client)

	// upload-url オペレーションでモッククライアントが呼ばれることを検証する。
	resp, err := b.Invoke(ctx, "upload-url", nil, map[string]string{"path": "uploads/test.txt"})
	require.NoError(t, err)
	assert.NotNil(t, resp)
	assert.NotEmpty(t, resp.Data)

	// モッククライアントが正しいパスで呼ばれたことを確認する。
	assert.Equal(t, 1, client.uploadURLCalled)
	assert.Equal(t, "uploads/test.txt", client.uploadPath)
}
