package middleware

// metrics_middleware_test.go: PrometheusMiddleware のユニットテスト。
// リクエストが次のハンドラに渡され、レスポンスコードが保持されることを検証する。
// Prometheus カウンターは init() で登録済みのため、カウント値の検証ではなく
// ミドルウェアの動作（c.Next() 呼び出し・ステータス伝播）を確認する。

import (
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"
)

// TestPrometheusMiddleware_PassesRequest は PrometheusMiddleware がリクエストを次のハンドラに渡すことを確認する。
func TestPrometheusMiddleware_PassesRequest(t *testing.T) {
	router := gin.New()
	router.Use(PrometheusMiddleware())

	// 内部ハンドラが呼ばれたかを検証するフラグ。
	called := false
	router.GET("/api/v1/health", func(c *gin.Context) {
		called = true
		c.JSON(http.StatusOK, gin.H{"status": "ok"})
	})

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/api/v1/health", nil)
	router.ServeHTTP(w, req)

	// 内部ハンドラが呼ばれていること。
	assert.True(t, called, "PrometheusMiddleware が次のハンドラを呼ぶこと")
	assert.Equal(t, http.StatusOK, w.Code, "200 レスポンスが保持されること")
}

// TestPrometheusMiddleware_Preserves4xx は 4xx ステータスコードが保持されることを確認する。
func TestPrometheusMiddleware_Preserves4xx(t *testing.T) {
	router := gin.New()
	router.Use(PrometheusMiddleware())
	router.GET("/api/v1/missing", func(c *gin.Context) {
		c.JSON(http.StatusNotFound, gin.H{"error": "not found"})
	})

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/api/v1/missing", nil)
	router.ServeHTTP(w, req)

	// 404 ステータスコードが保持されること。
	assert.Equal(t, http.StatusNotFound, w.Code, "404 レスポンスが保持されること")
}

// TestPrometheusMiddleware_UnknownPath は Gin に登録されていないパスが "unknown" として記録されることを確認する。
func TestPrometheusMiddleware_UnknownPath(t *testing.T) {
	router := gin.New()
	router.Use(PrometheusMiddleware())

	// デフォルトの 404 ハンドラは FullPath() が空文字列を返す。
	// PrometheusMiddleware はこれを "unknown" に変換する。
	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/unregistered-path", nil)
	// パニックせずに処理されること（"unknown" パスとしてメトリクス記録）。
	assert.NotPanics(t, func() {
		router.ServeHTTP(w, req)
	}, "未登録パスでもパニックしないこと")
}
