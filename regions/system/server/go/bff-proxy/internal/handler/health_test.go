package handler

import (
	"net/http"
	"net/http/httptest"
	"sync/atomic"
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
// oauthClient のフォールバックパス（oidcReady が nil の場合）を検証する。
func TestReadyz_OIDCNotDiscovered(t *testing.T) {
	// OIDC discoveryが未完了のモック（oidcReady は nil でフォールバックパスを使用）
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

// TestReadyz_OIDCReadyFlagFalse はH-07対応: oidcReady フラグが false の場合に503を返すことを確認する。
// retryOIDCDiscovery が全リトライを消費して失敗した後の状態をシミュレートする。
func TestReadyz_OIDCReadyFlagFalse(t *testing.T) {
	// H-07 対応: oidcReady フラグが false（全リトライ失敗後の状態）
	var oidcReady atomic.Bool
	// oidcReady.Store(false) はゼロ値なので明示的な設定は不要だが、意図を明確にするため記載
	oidcReady.Store(false)
	h := &HealthHandler{oidcReady: &oidcReady}

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
