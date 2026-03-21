package middleware

// security_headers_test.go: SecurityHeadersMiddleware のユニットテスト。
// 全セキュリティヘッダーが正しい値でレスポンスに付与されることを検証する。

import (
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"
)

// TestSecurityHeadersMiddleware_AllHeaders は全セキュリティヘッダーが付与されることを確認する。
func TestSecurityHeadersMiddleware_AllHeaders(t *testing.T) {
	router := gin.New()
	router.Use(SecurityHeadersMiddleware())
	router.GET("/test", func(c *gin.Context) {
		c.Status(http.StatusOK)
	})

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)

	// クリックジャッキング防止ヘッダーが設定されること。
	assert.Equal(t, "DENY", w.Header().Get("X-Frame-Options"),
		"X-Frame-Options が DENY であること")

	// MIMEスニッフィング防止ヘッダーが設定されること。
	assert.Equal(t, "nosniff", w.Header().Get("X-Content-Type-Options"),
		"X-Content-Type-Options が nosniff であること")

	// XSS 保護ヘッダーが設定されること（CSPで代替するため 0 を設定）。
	assert.Equal(t, "0", w.Header().Get("X-XSS-Protection"),
		"X-XSS-Protection が 0 であること")

	// HSTS ヘッダーが設定されること。
	assert.Equal(t, "max-age=31536000; includeSubDomains", w.Header().Get("Strict-Transport-Security"),
		"HSTS ヘッダーが正しく設定されること")

	// リファラポリシーが設定されること。
	assert.Equal(t, "strict-origin-when-cross-origin", w.Header().Get("Referrer-Policy"),
		"Referrer-Policy が strict-origin-when-cross-origin であること")

	// CSP が設定されること。
	assert.Equal(t, "default-src 'self'", w.Header().Get("Content-Security-Policy"),
		"CSP が default-src 'self' であること")
}

// TestSecurityHeadersMiddleware_CallsNext はセキュリティヘッダー付与後に次のハンドラが呼ばれることを確認する。
func TestSecurityHeadersMiddleware_CallsNext(t *testing.T) {
	router := gin.New()
	router.Use(SecurityHeadersMiddleware())

	// 内部ハンドラが呼ばれたかを検証するフラグ。
	called := false
	router.GET("/test", func(c *gin.Context) {
		called = true
		c.JSON(http.StatusCreated, gin.H{"ok": true})
	})

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	router.ServeHTTP(w, req)

	// ハンドラが呼ばれていること。
	assert.True(t, called, "セキュリティヘッダーミドルウェアが次のハンドラを呼ぶこと")
	// 内部ハンドラのステータスコードが保持されること。
	assert.Equal(t, http.StatusCreated, w.Code, "ハンドラのステータスコードが保持されること")
}
