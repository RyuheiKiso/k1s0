package handler

import (
	"context"
	"encoding/json"
	"log/slog"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/middleware"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/oauth"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/session"
)

// closeNotifierRecorder wraps httptest.ResponseRecorder with http.CloseNotifier
// to satisfy httputil.ReverseProxy requirements in tests.
type closeNotifierRecorder struct {
	*httptest.ResponseRecorder
}

func (c *closeNotifierRecorder) CloseNotify() <-chan bool {
	return make(chan bool)
}

// proxyTestStore is an in-memory store for proxy handler tests.
type proxyTestStore struct {
	sessions map[string]*session.SessionData
}

func newProxyTestStore() *proxyTestStore {
	return &proxyTestStore{sessions: make(map[string]*session.SessionData)}
}

func (s *proxyTestStore) Create(_ context.Context, data *session.SessionData, _ time.Duration) (string, error) {
	id := "proxy-session"
	s.sessions[id] = data
	return id, nil
}

func (s *proxyTestStore) Get(_ context.Context, id string) (*session.SessionData, error) {
	d, ok := s.sessions[id]
	if !ok {
		return nil, nil
	}
	return d, nil
}

func (s *proxyTestStore) Update(_ context.Context, id string, data *session.SessionData, _ time.Duration) error {
	s.sessions[id] = data
	return nil
}

func (s *proxyTestStore) Delete(_ context.Context, id string) error {
	delete(s.sessions, id)
	return nil
}

func (s *proxyTestStore) Touch(_ context.Context, _ string, _ time.Duration) error {
	return nil
}

func TestProxyHandler_InjectsAuthHeader(t *testing.T) {
	// Upstream server that verifies the Authorization header.
	upstream := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		assert.Equal(t, "Bearer test-access-token", r.Header.Get("Authorization"))
		w.WriteHeader(http.StatusOK)
		// errcheck: テストハンドラの書き込みエラーは無視する（§3.2 監査対応）
		_, _ = w.Write([]byte(`{"result": "ok"}`))
	}))
	defer upstream.Close()

	store := newProxyTestStore()
	store.sessions["test-session"] = &session.SessionData{
		AccessToken: "test-access-token",
		ExpiresAt:   time.Now().Add(10 * time.Minute).Unix(),
	}

	handler, err := NewProxyHandler(upstream.URL, store, nil, 30*time.Minute, 10*time.Second, nil)
	require.NoError(t, err)

	router := gin.New()
	router.Any("/api/*path", func(c *gin.Context) {
		c.Set(middleware.SessionDataKey, store.sessions["test-session"])
		c.Set(middleware.SessionIDKey, "test-session")
		handler.Handle(c)
	})

	rec := httptest.NewRecorder()
	w := &closeNotifierRecorder{rec}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/test", nil)
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, rec.Code)
}

// TestProxyHandler_RefreshFailure_DeletesSession はトークンリフレッシュ失敗時に
// セッションが削除されることを検証する（H-003）。
// 期限切れセッションのリフレッシュが失敗した場合、無効なセッションを再利用できないよう
// ストアから削除し、401 を返すことを確認する。
func TestProxyHandler_RefreshFailure_DeletesSession(t *testing.T) {
	// モック OIDC サーバー: discovery は成功するがトークンエンドポイントは常に失敗する
	oidcServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if strings.HasSuffix(r.URL.Path, "/.well-known/openid-configuration") {
			w.Header().Set("Content-Type", "application/json")
			// M-3 対応: 必須フィールド（issuer, authorization_endpoint, token_endpoint, jwks_uri）を全て含める
			_ = json.NewEncoder(w).Encode(map[string]string{
				"issuer":                 "http://" + r.Host,
				"authorization_endpoint": "http://" + r.Host + "/auth",
				"token_endpoint":         "http://" + r.Host + "/token",
				"jwks_uri":               "http://" + r.Host + "/jwks",
			})
			return
		}
		// /token エンドポイントは常に 401 を返してリフレッシュ失敗をシミュレートする
		w.WriteHeader(http.StatusUnauthorized)
		_, _ = w.Write([]byte(`{"error":"invalid_grant"}`))
	}))
	defer oidcServer.Close()

	// アップストリームサーバー（リフレッシュ失敗時は呼ばれないはず）
	upstreamServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		t.Fatal("リフレッシュ失敗時にアップストリームは呼ばれてはいけない")
	}))
	defer upstreamServer.Close()

	// oauthClient を discovery 済み状態で初期化する
	oauthClient := oauth.NewClient(
		context.Background(),
		oidcServer.URL,
		"test-client", "", "http://localhost/callback",
		[]string{"openid"},
		oauth.WithHTTPTimeout(5*time.Second),
	)
	_, err := oauthClient.Discover(context.Background())
	require.NoError(t, err)

	// 期限切れトークンを持つセッションをストアに登録する
	store := newProxyTestStore()
	store.sessions["refresh-session"] = &session.SessionData{
		AccessToken:  "expired-access-token",
		RefreshToken: "old-refresh-token",
		ExpiresAt:    time.Now().Add(-1 * time.Minute).Unix(),
	}

	// ログ出力をテストログに流すため slog.Default() を使用する（nil ロガーはパニックの原因になる）
	handler, err := NewProxyHandler(upstreamServer.URL, store, oauthClient, 30*time.Minute, 10*time.Second, slog.Default())
	require.NoError(t, err)

	router := gin.New()
	router.Any("/api/*path", func(c *gin.Context) {
		// SessionMiddleware が設定するのと同等のコンテキストキーを手動で設定する
		c.Set(middleware.SessionDataKey, store.sessions["refresh-session"])
		c.Set(middleware.SessionIDKey, "refresh-session")
		c.Set(middleware.SessionNeedsRefreshKey, true)
		handler.Handle(c)
	})

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/api/v1/test", nil)
	router.ServeHTTP(w, req)

	// リフレッシュ失敗時は 401 が返ること
	assert.Equal(t, http.StatusUnauthorized, w.Code)
	// リフレッシュ失敗後にセッションがストアから削除されていること（H-003）
	_, exists := store.sessions["refresh-session"]
	assert.False(t, exists, "リフレッシュ失敗後、無効なセッションはストアから削除されるべき")
}

func TestProxyHandler_NoSession(t *testing.T) {
	upstream := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		t.Fatal("upstream should not be called")
	}))
	defer upstream.Close()

	store := newProxyTestStore()
	handler, err := NewProxyHandler(upstream.URL, store, nil, 30*time.Minute, 10*time.Second, nil)
	require.NoError(t, err)

	router := gin.New()
	router.Any("/api/*path", handler.Handle)

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/api/v1/test", nil)
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusUnauthorized, w.Code)
}
