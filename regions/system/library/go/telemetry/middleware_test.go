package telemetry

// middleware_test.go: middleware.go のユニットテスト。
// HTTPMiddleware と GRPCUnaryInterceptor の正常な動作と
// ステータスコードの伝播を検証する。

import (
	"context"
	"errors"
	"log/slog"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestHTTPMiddleware_PassesRequest は HTTPMiddleware がリクエストをネストハンドラに渡すことを確認する。
func TestHTTPMiddleware_PassesRequest(t *testing.T) {
	logger := slog.Default()
	middleware := HTTPMiddleware(logger)

	// 内部ハンドラが呼ばれたかを検証するフラグ。
	called := false
	inner := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		called = true
		w.WriteHeader(http.StatusOK)
	})

	handler := middleware(inner)
	require.NotNil(t, handler)

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/api/test", nil)
	handler.ServeHTTP(w, req)

	// 内部ハンドラが呼ばれていること。
	assert.True(t, called, "内部ハンドラが呼ばれること")
	assert.Equal(t, http.StatusOK, w.Code, "レスポンスコードが正しく伝播すること")
}

// TestHTTPMiddleware_Captures4xxStatus は HTTPMiddleware が 4xx ステータスコードを正しく記録することを確認する。
func TestHTTPMiddleware_Captures4xxStatus(t *testing.T) {
	logger := slog.Default()
	middleware := HTTPMiddleware(logger)

	// 404 を返すハンドラを定義する。
	inner := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusNotFound)
	})

	handler := middleware(inner)
	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/not-found", nil)
	handler.ServeHTTP(w, req)

	// ミドルウェアが 404 レスポンスを正しくラップすること。
	assert.Equal(t, http.StatusNotFound, w.Code, "404 ステータスが保持されること")
}

// TestHTTPMiddleware_DefaultStatus は WriteHeader を呼ばない場合に 200 がデフォルトになることを確認する。
func TestHTTPMiddleware_DefaultStatus(t *testing.T) {
	logger := slog.Default()
	middleware := HTTPMiddleware(logger)

	// WriteHeader を呼ばないハンドラ（デフォルト 200）。
	inner := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// WriteHeader を呼ばない。
	})

	handler := middleware(inner)
	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodPost, "/api/data", nil)
	handler.ServeHTTP(w, req)

	// WriteHeader を省略した場合のデフォルトステータスが 200 であること。
	assert.Equal(t, http.StatusOK, w.Code, "デフォルトステータスが 200 であること")
}

// TestGRPCUnaryInterceptor_Success は GRPCUnaryInterceptor が成功時に invoker を呼び出すことを確認する。
func TestGRPCUnaryInterceptor_Success(t *testing.T) {
	logger := slog.Default()
	interceptor := GRPCUnaryInterceptor(logger)
	require.NotNil(t, interceptor)

	// 成功を返す invoker スタブ。
	called := false
	// gRPC unary invoker スタブ（interface{} → any: Go 1.18+ 推奨エイリアスを使用する）
	invoker := func(ctx context.Context, method string, req, reply any) error {
		called = true
		return nil
	}

	err := interceptor(context.Background(), "/AuthService/Login", nil, nil, invoker)

	assert.NoError(t, err, "成功時にエラーが返らないこと")
	assert.True(t, called, "invoker が呼ばれること")
}

// TestGRPCUnaryInterceptor_Error は GRPCUnaryInterceptor がエラー時に invoker のエラーを伝播することを確認する。
func TestGRPCUnaryInterceptor_Error(t *testing.T) {
	logger := slog.Default()
	interceptor := GRPCUnaryInterceptor(logger)

	// エラーを返す invoker スタブ（interface{} → any: Go 1.18+ 推奨エイリアスを使用する）。
	expectedErr := errors.New("gRPC: service unavailable")
	invoker := func(ctx context.Context, method string, req, reply any) error {
		return expectedErr
	}

	err := interceptor(context.Background(), "/TaskService/CreateTask", nil, nil, invoker)

	// invoker のエラーがそのまま返ること。
	assert.ErrorIs(t, err, expectedErr, "invoker のエラーが伝播すること")
}
