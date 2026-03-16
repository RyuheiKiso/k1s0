package servercommon

import (
	"net/http"
	"net/http/httptest"
	"testing"
)

// Chain が複数のミドルウェアを正しい順序で適用することを確認する。
func TestChainOrder(t *testing.T) {
	var order []string

	// 各ミドルウェアが呼び出し順を記録する
	mw1 := func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			order = append(order, "mw1-before")
			next.ServeHTTP(w, r)
			order = append(order, "mw1-after")
		})
	}

	mw2 := func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			order = append(order, "mw2-before")
			next.ServeHTTP(w, r)
			order = append(order, "mw2-after")
		})
	}

	handler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		order = append(order, "handler")
	})

	chained := Chain(mw1, mw2)(handler)
	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	w := httptest.NewRecorder()
	chained.ServeHTTP(w, req)

	// mw1 が最外側なので先に実行される
	expected := []string{"mw1-before", "mw2-before", "handler", "mw2-after", "mw1-after"}
	if len(order) != len(expected) {
		t.Fatalf("expected %d calls, got %d", len(expected), len(order))
	}
	for i, v := range expected {
		if order[i] != v {
			t.Errorf("position %d: expected '%s', got '%s'", i, v, order[i])
		}
	}
}

// Chain に空のミドルウェアスライスを渡してもハンドラーが正常に動作することを確認する。
func TestChainEmpty(t *testing.T) {
	called := false
	handler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		called = true
	})

	chained := Chain()(handler)
	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	w := httptest.NewRecorder()
	chained.ServeHTTP(w, req)

	if !called {
		t.Error("handler was not called with empty chain")
	}
}

// RequestIDMiddleware がリクエストに X-Request-ID を付与することを確認する。
func TestRequestIDMiddlewareGeneratesID(t *testing.T) {
	handler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	})

	wrapped := RequestIDMiddleware(handler)
	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	w := httptest.NewRecorder()
	wrapped.ServeHTTP(w, req)

	// レスポンスヘッダーに X-Request-ID が設定されていることを確認する
	requestID := w.Header().Get("X-Request-ID")
	if requestID == "" {
		t.Error("expected X-Request-ID header to be set")
	}
}

// 既存の X-Request-ID がある場合にそのまま使用されることを確認する。
func TestRequestIDMiddlewarePreservesExistingID(t *testing.T) {
	handler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	})

	wrapped := RequestIDMiddleware(handler)
	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	req.Header.Set("X-Request-ID", "existing-id-123")
	w := httptest.NewRecorder()
	wrapped.ServeHTTP(w, req)

	requestID := w.Header().Get("X-Request-ID")
	if requestID != "existing-id-123" {
		t.Errorf("expected X-Request-ID 'existing-id-123', got '%s'", requestID)
	}
}

// CORSMiddleware が許可されたオリジンに対して CORS ヘッダーを設定することを確認する。
func TestCORSMiddlewareAllowedOrigin(t *testing.T) {
	handler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	})

	cors := CORSMiddleware([]string{"https://example.com", "https://app.example.com"})
	wrapped := cors(handler)

	req := httptest.NewRequest(http.MethodGet, "/api/data", nil)
	req.Header.Set("Origin", "https://example.com")
	w := httptest.NewRecorder()
	wrapped.ServeHTTP(w, req)

	// Access-Control-Allow-Origin が設定されていることを確認する
	origin := w.Header().Get("Access-Control-Allow-Origin")
	if origin != "https://example.com" {
		t.Errorf("expected origin 'https://example.com', got '%s'", origin)
	}

	// 許可メソッドヘッダーが設定されていることを確認する
	methods := w.Header().Get("Access-Control-Allow-Methods")
	if methods == "" {
		t.Error("expected Access-Control-Allow-Methods header to be set")
	}
}

// CORSMiddleware が許可されていないオリジンに対して Access-Control-Allow-Origin を設定しないことを確認する。
func TestCORSMiddlewareDisallowedOrigin(t *testing.T) {
	handler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	})

	cors := CORSMiddleware([]string{"https://example.com"})
	wrapped := cors(handler)

	req := httptest.NewRequest(http.MethodGet, "/api/data", nil)
	req.Header.Set("Origin", "https://malicious.com")
	w := httptest.NewRecorder()
	wrapped.ServeHTTP(w, req)

	origin := w.Header().Get("Access-Control-Allow-Origin")
	if origin != "" {
		t.Errorf("expected no Access-Control-Allow-Origin, got '%s'", origin)
	}
}

// CORSMiddleware がワイルドカード "*" で全オリジンを許可することを確認する。
func TestCORSMiddlewareWildcard(t *testing.T) {
	handler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	})

	cors := CORSMiddleware([]string{"*"})
	wrapped := cors(handler)

	req := httptest.NewRequest(http.MethodGet, "/api/data", nil)
	req.Header.Set("Origin", "https://any-origin.com")
	w := httptest.NewRecorder()
	wrapped.ServeHTTP(w, req)

	origin := w.Header().Get("Access-Control-Allow-Origin")
	if origin != "https://any-origin.com" {
		t.Errorf("expected origin 'https://any-origin.com', got '%s'", origin)
	}
}

// CORSMiddleware が OPTIONS プリフライトリクエストに 204 を返すことを確認する。
func TestCORSMiddlewarePreflight(t *testing.T) {
	called := false
	handler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		called = true
	})

	cors := CORSMiddleware([]string{"*"})
	wrapped := cors(handler)

	req := httptest.NewRequest(http.MethodOptions, "/api/data", nil)
	req.Header.Set("Origin", "https://example.com")
	w := httptest.NewRecorder()
	wrapped.ServeHTTP(w, req)

	if w.Code != http.StatusNoContent {
		t.Errorf("expected status 204 for OPTIONS, got %d", w.Code)
	}

	// ハンドラーが呼ばれないことを確認する
	if called {
		t.Error("handler should not be called for OPTIONS preflight")
	}
}

// RecoveryMiddleware がパニック時に 500 を返すことを確認する。
func TestRecoveryMiddlewareCatchesPanic(t *testing.T) {
	handler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		panic("something went wrong")
	})

	wrapped := RecoveryMiddleware(handler)
	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	w := httptest.NewRecorder()
	wrapped.ServeHTTP(w, req)

	if w.Code != http.StatusInternalServerError {
		t.Errorf("expected status 500 on panic, got %d", w.Code)
	}
}

// RecoveryMiddleware がパニックなしの場合に正常にレスポンスを返すことを確認する。
func TestRecoveryMiddlewareNoPanic(t *testing.T) {
	handler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	})

	wrapped := RecoveryMiddleware(handler)
	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	w := httptest.NewRecorder()
	wrapped.ServeHTTP(w, req)

	if w.Code != http.StatusOK {
		t.Errorf("expected status 200, got %d", w.Code)
	}
}

// generateID が空でない文字列を返すことを確認する。
func TestGenerateIDNotEmpty(t *testing.T) {
	id := generateID()
	if id == "" {
		t.Error("expected non-empty ID")
	}
}

// generateID が時間ベースの ID を生成し、一定の形式を持つことを確認する。
func TestGenerateIDFormat(t *testing.T) {
	id := generateID()
	// "20060102150405.000000000" の形式で 24 文字以上になることを確認する
	if len(id) < 20 {
		t.Errorf("expected ID with at least 20 characters, got %d: '%s'", len(id), id)
	}
	// ドット区切りがあることを確認する（日付部分.ナノ秒部分）
	found := false
	for _, c := range id {
		if c == '.' {
			found = true
			break
		}
	}
	if !found {
		t.Errorf("expected ID to contain a dot separator, got '%s'", id)
	}
}
