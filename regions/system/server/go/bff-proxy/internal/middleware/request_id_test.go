package middleware

// request_id_test.go: GetRequestID 関数のユニットテスト。
// Gin コンテキストからリクエスト ID を正しく取得できることを検証する。

import (
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"
)

// TestGetRequestID_FromContextKey は Gin コンテキストのキーからリクエスト ID を取得できることを確認する。
func TestGetRequestID_FromContextKey(t *testing.T) {
	router := gin.New()
	router.GET("/test", func(c *gin.Context) {
		// コンテキストに correlation ID をセットして GetRequestID で取得する。
		c.Set(CorrelationIDKey, "test-correlation-id-123")
		id := GetRequestID(c)
		assert.Equal(t, "test-correlation-id-123", id,
			"コンテキストキーからリクエスト ID が取得されること")
		c.Status(http.StatusOK)
	})

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)
}

// TestGetRequestID_FallbackToHeader はコンテキストキーがない場合にヘッダーからリクエスト ID を取得できることを確認する。
func TestGetRequestID_FallbackToHeader(t *testing.T) {
	router := gin.New()
	router.GET("/test", func(c *gin.Context) {
		// コンテキストキーをセットせずにヘッダーのみを使用する。
		id := GetRequestID(c)
		assert.Equal(t, "header-correlation-id-456", id,
			"ヘッダーからリクエスト ID が取得されること")
		c.Status(http.StatusOK)
	})

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	// ヘッダーに correlation ID をセットする。
	req.Header.Set(HeaderCorrelationID, "header-correlation-id-456")
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)
}

// TestGetRequestID_Empty はコンテキストキーもヘッダーもない場合に空文字列が返ることを確認する。
func TestGetRequestID_Empty(t *testing.T) {
	router := gin.New()
	router.GET("/test", func(c *gin.Context) {
		// コンテキストキーもヘッダーも設定しない場合。
		id := GetRequestID(c)
		assert.Empty(t, id, "ID が存在しない場合に空文字列が返ること")
		c.Status(http.StatusOK)
	})

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)
}

// TestGetRequestID_ContextKeyTakesPrecedence はコンテキストキーがヘッダーより優先されることを確認する。
func TestGetRequestID_ContextKeyTakesPrecedence(t *testing.T) {
	router := gin.New()
	router.GET("/test", func(c *gin.Context) {
		// コンテキストキーとヘッダーの両方をセットした場合、コンテキストキーが優先される。
		c.Set(CorrelationIDKey, "context-key-id")
		id := GetRequestID(c)
		assert.Equal(t, "context-key-id", id,
			"コンテキストキーがヘッダーより優先されること")
		c.Status(http.StatusOK)
	})

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	req.Header.Set(HeaderCorrelationID, "header-id")
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)
}
