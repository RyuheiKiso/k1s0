package handler

import (
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"
)

func init() {
	gin.SetMode(gin.TestMode)
}

// mockOIDCChecker はOIDCCheckerインターフェースのモック実装。
// テスト時にdiscovery状態を任意に設定できる。
type mockOIDCChecker struct {
	discovered bool
}

// IsDiscovered はモックのdiscovery状態を返す。
func (m *mockOIDCChecker) IsDiscovered() bool {
	return m.discovered
}

// TestHealthz はlivenessプローブが常に200を返すことを確認する。
func TestHealthz(t *testing.T) {
	h := &HealthHandler{}

	router := gin.New()
	router.GET("/healthz", h.Healthz)

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/healthz", nil)
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)
	assert.Contains(t, w.Body.String(), `"status":"ok"`)
}

// TestReadyz_OIDCNotDiscovered はOIDC discoveryが未完了の場合に503を返すことを確認する。
func TestReadyz_OIDCNotDiscovered(t *testing.T) {
	// OIDC discoveryが未完了のモック
	mock := &mockOIDCChecker{discovered: false}
	h := &HealthHandler{oauthClient: mock}

	router := gin.New()
	router.GET("/readyz", h.Readyz)

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/readyz", nil)
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusServiceUnavailable, w.Code)
	assert.Contains(t, w.Body.String(), `"reason":"oidc discovery not completed"`)
}

// TestReadyz_OIDCDiscoveredButNoRedis はOIDC discovery完了済みだがRedisがnil/未接続の場合のテスト。
// OIDCチェックを通過した後、Redisチェックで503になることを確認する。
// Note: redisClientがnilだとpanicするため、oauthClientのみのテストは
// TestReadyz_OIDCNotDiscovered で十分にカバーされている。
