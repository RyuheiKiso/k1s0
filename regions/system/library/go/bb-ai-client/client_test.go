package bbaiclient_test

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	bbaiclient "github.com/k1s0-platform/system-library-go-bb-ai-client"
)

// newTestServer はテスト用の HTTP サーバーを起動し、リクエストを検証する。
// handler で任意のレスポンスを返せるため、各テストケースで使い回せる。
func newTestServer(t *testing.T, handler http.HandlerFunc) *httptest.Server {
	t.Helper()
	return httptest.NewServer(handler)
}

// TestHTTPClient_Complete はチャット補完エンドポイントの正常系を検証する。
func TestHTTPClient_Complete(t *testing.T) {
	// テスト用レスポンスを定義
	want := bbaiclient.CompleteResponse{
		ID:      "resp-1",
		Model:   "gpt-4o",
		Content: "Hello, world!",
		Usage:   bbaiclient.Usage{InputTokens: 10, OutputTokens: 5},
	}

	// ダミーサーバーを起動して期待レスポンスを返す
	srv := newTestServer(t, func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			t.Errorf("メソッドが不正: got %s, want POST", r.Method)
		}
		if r.URL.Path != "/v1/complete" {
			t.Errorf("パスが不正: got %s, want /v1/complete", r.URL.Path)
		}
		w.Header().Set("Content-Type", "application/json")
		if err := json.NewEncoder(w).Encode(want); err != nil {
			t.Fatalf("レスポンスのエンコードに失敗: %v", err)
		}
	})
	defer srv.Close()

	client := bbaiclient.NewHTTPClient(bbaiclient.HTTPClientConfig{
		BaseURL: srv.URL,
		Timeout: 5 * time.Second,
	})

	got, err := client.Complete(context.Background(), bbaiclient.CompleteRequest{
		Model:    "gpt-4o",
		Messages: []bbaiclient.ChatMessage{{Role: "user", Content: "Hello"}},
	})
	if err != nil {
		t.Fatalf("Complete() エラー: %v", err)
	}
	if got.ID != want.ID || got.Content != want.Content {
		t.Errorf("Complete() = %+v, want %+v", got, want)
	}
}

// TestHTTPClient_Complete_Error はサーバーエラー時に APIError が返ることを検証する。
func TestHTTPClient_Complete_Error(t *testing.T) {
	// 500 エラーを返すサーバー
	srv := newTestServer(t, func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusInternalServerError)
	})
	defer srv.Close()

	client := bbaiclient.NewHTTPClient(bbaiclient.HTTPClientConfig{
		BaseURL: srv.URL,
		Timeout: 5 * time.Second,
	})

	_, err := client.Complete(context.Background(), bbaiclient.CompleteRequest{
		Model:    "gpt-4o",
		Messages: []bbaiclient.ChatMessage{{Role: "user", Content: "Hello"}},
	})
	if err == nil {
		t.Fatal("エラーが期待されましたが nil でした")
	}
	apiErr, ok := err.(*bbaiclient.APIError)
	if !ok {
		t.Fatalf("エラー型が不正: got %T, want *bbaiclient.APIError", err)
	}
	if apiErr.StatusCode != http.StatusInternalServerError {
		t.Errorf("StatusCode = %d, want %d", apiErr.StatusCode, http.StatusInternalServerError)
	}
}

// TestHTTPClient_Embed はテキスト埋め込みエンドポイントの正常系を検証する。
func TestHTTPClient_Embed(t *testing.T) {
	want := bbaiclient.EmbedResponse{
		Model:      "text-embedding-3-small",
		Embeddings: [][]float64{{0.1, 0.2, 0.3}},
	}

	srv := newTestServer(t, func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/v1/embed" {
			t.Errorf("パスが不正: got %s, want /v1/embed", r.URL.Path)
		}
		w.Header().Set("Content-Type", "application/json")
		if err := json.NewEncoder(w).Encode(want); err != nil {
			t.Fatalf("レスポンスのエンコードに失敗: %v", err)
		}
	})
	defer srv.Close()

	client := bbaiclient.NewHTTPClient(bbaiclient.HTTPClientConfig{
		BaseURL: srv.URL,
		Timeout: 5 * time.Second,
	})

	got, err := client.Embed(context.Background(), bbaiclient.EmbedRequest{
		Model: "text-embedding-3-small",
		Texts: []string{"hello"},
	})
	if err != nil {
		t.Fatalf("Embed() エラー: %v", err)
	}
	if got.Model != want.Model || len(got.Embeddings) != len(want.Embeddings) {
		t.Errorf("Embed() = %+v, want %+v", got, want)
	}
}

// TestHTTPClient_ListModels はモデル一覧エンドポイントの正常系を検証する。
func TestHTTPClient_ListModels(t *testing.T) {
	want := []bbaiclient.ModelInfo{
		{ID: "gpt-4o", Name: "GPT-4o", Description: "最新のマルチモーダルモデル"},
	}

	srv := newTestServer(t, func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/v1/models" {
			t.Errorf("パスが不正: got %s, want /v1/models", r.URL.Path)
		}
		if r.Method != http.MethodGet {
			t.Errorf("メソッドが不正: got %s, want GET", r.Method)
		}
		w.Header().Set("Content-Type", "application/json")
		if err := json.NewEncoder(w).Encode(want); err != nil {
			t.Fatalf("レスポンスのエンコードに失敗: %v", err)
		}
	})
	defer srv.Close()

	client := bbaiclient.NewHTTPClient(bbaiclient.HTTPClientConfig{
		BaseURL: srv.URL,
		Timeout: 5 * time.Second,
	})

	got, err := client.ListModels(context.Background())
	if err != nil {
		t.Fatalf("ListModels() エラー: %v", err)
	}
	if len(got) != len(want) || got[0].ID != want[0].ID {
		t.Errorf("ListModels() = %+v, want %+v", got, want)
	}
}

// TestHTTPClient_APIKey は APIKey ヘッダーが正しく付与されることを検証する。
func TestHTTPClient_APIKey(t *testing.T) {
	const testKey = "test-secret-key"

	srv := newTestServer(t, func(w http.ResponseWriter, r *http.Request) {
		auth := r.Header.Get("Authorization")
		if auth != "Bearer "+testKey {
			t.Errorf("Authorization ヘッダーが不正: got %s, want Bearer %s", auth, testKey)
		}
		w.Header().Set("Content-Type", "application/json")
		if err := json.NewEncoder(w).Encode(bbaiclient.CompleteResponse{ID: "ok"}); err != nil {
			t.Fatalf("レスポンスのエンコードに失敗: %v", err)
		}
	})
	defer srv.Close()

	client := bbaiclient.NewHTTPClient(bbaiclient.HTTPClientConfig{
		BaseURL: srv.URL,
		APIKey:  testKey,
		Timeout: 5 * time.Second,
	})

	_, err := client.Complete(context.Background(), bbaiclient.CompleteRequest{
		Model:    "gpt-4o",
		Messages: []bbaiclient.ChatMessage{{Role: "user", Content: "test"}},
	})
	if err != nil {
		t.Fatalf("Complete() エラー: %v", err)
	}
}

// TestMockClient_Complete はモッククライアントのデフォルト動作を検証する。
func TestMockClient_Complete(t *testing.T) {
	mock := &bbaiclient.MockClient{}

	resp, err := mock.Complete(context.Background(), bbaiclient.CompleteRequest{
		Model:    "mock-model",
		Messages: []bbaiclient.ChatMessage{{Role: "user", Content: "hi"}},
	})
	if err != nil {
		t.Fatalf("MockClient.Complete() エラー: %v", err)
	}
	if resp.ID != "mock" {
		t.Errorf("MockClient.Complete().ID = %s, want mock", resp.ID)
	}
}

// TestMockClient_Embed はモッククライアントの埋め込みデフォルト動作を検証する。
func TestMockClient_Embed(t *testing.T) {
	mock := &bbaiclient.MockClient{}

	resp, err := mock.Embed(context.Background(), bbaiclient.EmbedRequest{
		Model: "mock-embed",
		Texts: []string{"test"},
	})
	if err != nil {
		t.Fatalf("MockClient.Embed() エラー: %v", err)
	}
	if len(resp.Embeddings) == 0 {
		t.Error("MockClient.Embed().Embeddings が空です")
	}
}

// TestMockClient_ListModels はモッククライアントのモデル一覧デフォルト動作を検証する。
func TestMockClient_ListModels(t *testing.T) {
	mock := &bbaiclient.MockClient{}

	models, err := mock.ListModels(context.Background())
	if err != nil {
		t.Fatalf("MockClient.ListModels() エラー: %v", err)
	}
	if len(models) == 0 {
		t.Error("MockClient.ListModels() が空です")
	}
}

// TestMockClient_CustomFn はカスタムハンドラが正しく呼び出されることを検証する。
func TestMockClient_CustomFn(t *testing.T) {
	mock := &bbaiclient.MockClient{
		CompleteFn: func(_ context.Context, req bbaiclient.CompleteRequest) (*bbaiclient.CompleteResponse, error) {
			// カスタムハンドラでモデル名をエコーバックする
			return &bbaiclient.CompleteResponse{ID: "custom", Model: req.Model, Content: "カスタム応答"}, nil
		},
	}

	resp, err := mock.Complete(context.Background(), bbaiclient.CompleteRequest{
		Model:    "custom-model",
		Messages: []bbaiclient.ChatMessage{{Role: "user", Content: "test"}},
	})
	if err != nil {
		t.Fatalf("MockClient.Complete() エラー: %v", err)
	}
	if resp.Content != "カスタム応答" {
		t.Errorf("MockClient.Complete().Content = %s, want カスタム応答", resp.Content)
	}
}

// TestAPIError_Error はエラーメッセージのフォーマットを検証する。
func TestAPIError_Error(t *testing.T) {
	tests := []struct {
		name    string
		err     *bbaiclient.APIError
		wantMsg string
	}{
		{
			name:    "メッセージなし",
			err:     &bbaiclient.APIError{StatusCode: 404},
			wantMsg: "bbaiclient: API error 404",
		},
		{
			name:    "メッセージあり",
			err:     &bbaiclient.APIError{StatusCode: 401, Message: "Unauthorized"},
			wantMsg: "bbaiclient: API error 401: Unauthorized",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := tt.err.Error(); got != tt.wantMsg {
				t.Errorf("APIError.Error() = %q, want %q", got, tt.wantMsg)
			}
		})
	}
}
