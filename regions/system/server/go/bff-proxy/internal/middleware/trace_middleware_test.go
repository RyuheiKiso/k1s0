// trace_middleware_test.go — CorrelationMiddleware のトレース ID 伝播テスト。
// correlation_test.go の基本テストを補完し、X-Trace-Id ヘッダーの伝播と
// 両ヘッダー同時設定のシナリオを検証する。
package middleware

import (
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"
)

// TestCorrelationMiddleware_PropagatesExistingTraceID は既存 X-Trace-Id の伝播テスト。
// リクエストに X-Trace-Id が含まれる場合、同じ値がレスポンスヘッダーに設定されることを検証する。
func TestCorrelationMiddleware_PropagatesExistingTraceID(t *testing.T) {
	router := gin.New()
	router.Use(CorrelationMiddleware())
	router.GET("/test", func(c *gin.Context) {
		tid, _ := c.Get(TraceIDKey)
		assert.Equal(t, "existing-trace-id", tid)
		c.Status(http.StatusOK)
	})

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	req.Header.Set(HeaderTraceID, "existing-trace-id")
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)
	assert.Equal(t, "existing-trace-id", w.Header().Get(HeaderTraceID))
}

// TestCorrelationMiddleware_BothHeadersSet は CorrelationID と TraceID の両方が同時に設定される場合のテスト。
// 両ヘッダーがリクエストに含まれる場合、それぞれの値が保持されることを検証する。
func TestCorrelationMiddleware_BothHeadersSet(t *testing.T) {
	router := gin.New()
	router.Use(CorrelationMiddleware())
	router.GET("/test", func(c *gin.Context) {
		cid, _ := c.Get(CorrelationIDKey)
		tid, _ := c.Get(TraceIDKey)
		assert.Equal(t, "my-correlation-id", cid)
		assert.Equal(t, "my-trace-id", tid)
		c.Status(http.StatusOK)
	})

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	req.Header.Set(HeaderCorrelationID, "my-correlation-id")
	req.Header.Set(HeaderTraceID, "my-trace-id")
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)
	assert.Equal(t, "my-correlation-id", w.Header().Get(HeaderCorrelationID))
	assert.Equal(t, "my-trace-id", w.Header().Get(HeaderTraceID))
}

// TestCorrelationMiddleware_GeneratesUniqueIDs は連続リクエストで異なるトレース ID が生成されることを検証する。
// 各リクエストに一意の相関 ID とトレース ID が割り当てられることを確認する。
func TestCorrelationMiddleware_GeneratesUniqueIDs(t *testing.T) {
	router := gin.New()
	router.Use(CorrelationMiddleware())
	router.GET("/test", func(c *gin.Context) {
		c.Status(http.StatusOK)
	})

	ids := make(map[string]bool)
	const requestCount = 10
	for range requestCount {
		w := httptest.NewRecorder()
		req := httptest.NewRequest(http.MethodGet, "/test", nil)
		router.ServeHTTP(w, req)

		cid := w.Header().Get(HeaderCorrelationID)
		tid := w.Header().Get(HeaderTraceID)
		assert.NotEmpty(t, cid)
		assert.NotEmpty(t, tid)
		ids[cid] = true
		ids[tid] = true
	}

	// 全 ID が一意であること（20 個すべて異なる値）
	assert.Len(t, ids, requestCount*2, "each request should generate unique correlation and trace IDs")
}

// TestGenerateTraceID_IsLowerHex はトレース ID が小文字 16 進数のみで構成されることを検証する。
// W3C Trace Context 仕様との互換性のため、大文字を含まないことを確認する。
func TestGenerateTraceID_IsLowerHex(t *testing.T) {
	for range 100 {
		id := generateTraceID()
		assert.Len(t, id, 32)
		assert.Equal(t, strings.ToLower(id), id, "trace ID should be lowercase hex")
		assert.NotContains(t, id, "-", "trace ID should not contain hyphens")
	}
}

// TestCorrelationMiddleware_ContextKeysSet はミドルウェアが gin.Context にキーを正しく設定することを検証する。
// CorrelationIDKey と TraceIDKey がコンテキストで文字列として取得できることを確認する。
func TestCorrelationMiddleware_ContextKeysSet(t *testing.T) {
	router := gin.New()
	router.Use(CorrelationMiddleware())
	router.GET("/test", func(c *gin.Context) {
		// GetString でコンテキストキーが文字列として取得できること
		cid := c.GetString(CorrelationIDKey)
		tid := c.GetString(TraceIDKey)
		assert.NotEmpty(t, cid, "correlation_id should be set in context")
		assert.NotEmpty(t, tid, "trace_id should be set in context")
		assert.Len(t, tid, 32, "auto-generated trace ID should be 32 chars")
		c.Status(http.StatusOK)
	})

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)
}
