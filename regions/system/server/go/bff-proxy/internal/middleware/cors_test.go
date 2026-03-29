package middleware

import (
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/config"
)

// requiresCredentials のユニットテスト（H-13 監査対応）
func TestRequiresCredentials(t *testing.T) {
	tests := []struct {
		name             string
		path             string
		credentialsPaths []string
		want             bool
	}{
		{
			// HIGH-1 監査対応: credentialsPaths が未設定の場合は最小権限原則に従い false を返す
			// LOW-07 監査対応でコード変更済み（空→falseを返す）のため期待値を修正する
			name:             "credentialsPaths が空の場合は最小権限原則でfalseを返す",
			path:             "/healthz",
			credentialsPaths: []string{},
			want:             false,
		},
		{
			// HIGH-1 監査対応: nil の場合も同様に false を返す（credentialsパスが未設定状態）
			name:             "credentialsPaths が nil の場合は最小権限原則でfalseを返す",
			path:             "/metrics",
			credentialsPaths: nil,
			want:             false,
		},
		{
			name:             "/auth/ に一致するパスはtrueを返す",
			path:             "/auth/login",
			credentialsPaths: []string{"/auth/", "/api/"},
			want:             true,
		},
		{
			name:             "/api/ に一致するパスはtrueを返す",
			path:             "/api/v1/users",
			credentialsPaths: []string{"/auth/", "/api/"},
			want:             true,
		},
		{
			name:             "一致しないパスはfalseを返す",
			path:             "/healthz",
			credentialsPaths: []string{"/auth/", "/api/"},
			want:             false,
		},
		{
			name:             "/metrics はfalseを返す",
			path:             "/metrics",
			credentialsPaths: []string{"/auth/", "/api/"},
			want:             false,
		},
		{
			name:             "プレフィックスの部分一致はfalseを返す",
			path:             "/public/assets",
			credentialsPaths: []string{"/auth/", "/api/"},
			want:             false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := requiresCredentials(tt.path, tt.credentialsPaths)
			assert.Equal(t, tt.want, got)
		})
	}
}

// CORSMiddleware の統合テスト（H-13 監査対応）
func TestCORSMiddleware_CredentialsPerEndpoint(t *testing.T) {
	gin.SetMode(gin.TestMode)

	cfg := config.CORSConfig{
		Enabled:      true,
		AllowOrigins: []string{"http://localhost:3000"},
		CredentialsPaths: []string{
			"/auth/",
			"/api/",
		},
		MaxAgeSecs: 600,
	}

	handler, err := CORSMiddleware(cfg)
	require.NoError(t, err)

	router := gin.New()
	router.Use(handler)
	router.GET("/auth/login", func(c *gin.Context) { c.Status(http.StatusOK) })
	router.GET("/api/v1/users", func(c *gin.Context) { c.Status(http.StatusOK) })
	router.GET("/healthz", func(c *gin.Context) { c.Status(http.StatusOK) })
	router.GET("/metrics", func(c *gin.Context) { c.Status(http.StatusOK) })

	t.Run("認証エンドポイントにはCredentials=trueが付与される", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodGet, "/auth/login", nil)
		req.Header.Set("Origin", "http://localhost:3000")
		w := httptest.NewRecorder()
		router.ServeHTTP(w, req)

		assert.Equal(t, "http://localhost:3000", w.Header().Get("Access-Control-Allow-Origin"))
		assert.Equal(t, "true", w.Header().Get("Access-Control-Allow-Credentials"))
	})

	t.Run("APIエンドポイントにはCredentials=trueが付与される", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodGet, "/api/v1/users", nil)
		req.Header.Set("Origin", "http://localhost:3000")
		w := httptest.NewRecorder()
		router.ServeHTTP(w, req)

		assert.Equal(t, "http://localhost:3000", w.Header().Get("Access-Control-Allow-Origin"))
		assert.Equal(t, "true", w.Header().Get("Access-Control-Allow-Credentials"))
	})

	t.Run("healthzにはCredentialsが付与されない", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodGet, "/healthz", nil)
		req.Header.Set("Origin", "http://localhost:3000")
		w := httptest.NewRecorder()
		router.ServeHTTP(w, req)

		assert.Equal(t, "http://localhost:3000", w.Header().Get("Access-Control-Allow-Origin"))
		assert.Empty(t, w.Header().Get("Access-Control-Allow-Credentials"))
	})

	t.Run("metricsにはCredentialsが付与されない", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodGet, "/metrics", nil)
		req.Header.Set("Origin", "http://localhost:3000")
		w := httptest.NewRecorder()
		router.ServeHTTP(w, req)

		assert.Equal(t, "http://localhost:3000", w.Header().Get("Access-Control-Allow-Origin"))
		assert.Empty(t, w.Header().Get("Access-Control-Allow-Credentials"))
	})

	t.Run("許可されていないオリジンにはCORSヘッダーが付与されない", func(t *testing.T) {
		req := httptest.NewRequest(http.MethodGet, "/auth/login", nil)
		req.Header.Set("Origin", "http://evil.example.com")
		w := httptest.NewRecorder()
		router.ServeHTTP(w, req)

		assert.Empty(t, w.Header().Get("Access-Control-Allow-Origin"))
		assert.Empty(t, w.Header().Get("Access-Control-Allow-Credentials"))
	})
}

// ワイルドカード拒否テスト
func TestCORSMiddleware_RejectsWildcard(t *testing.T) {
	gin.SetMode(gin.TestMode)

	cfg := config.CORSConfig{
		Enabled:      true,
		AllowOrigins: []string{"*"},
	}

	_, err := CORSMiddleware(cfg)
	assert.Error(t, err, "ワイルドカード指定はエラーになること")
}
