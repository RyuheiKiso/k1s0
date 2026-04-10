package middleware

import (
	"context"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/session"
)

// testStore is an in-memory session store for middleware tests.
type testStore struct {
	sessions map[string]*session.Data
	touched  map[string]time.Duration
}

func newTestStore() *testStore {
	return &testStore{
		sessions: make(map[string]*session.Data),
		touched:  make(map[string]time.Duration),
	}
}

func (s *testStore) Create(_ context.Context, data *session.Data, _ time.Duration) (string, error) {
	id := "test-id"
	s.sessions[id] = data
	return id, nil
}

func (s *testStore) Get(_ context.Context, id string) (*session.Data, error) {
	d, ok := s.sessions[id]
	if !ok {
		return nil, nil
	}
	return d, nil
}

func (s *testStore) Update(_ context.Context, id string, data *session.Data, _ time.Duration) error {
	s.sessions[id] = data
	return nil
}

func (s *testStore) Delete(_ context.Context, id string) error {
	delete(s.sessions, id)
	return nil
}

func (s *testStore) Touch(_ context.Context, id string, ttl time.Duration) error {
	s.touched[id] = ttl
	return nil
}

func TestSessionMiddleware_MissingCookie(t *testing.T) {
	store := newTestStore()
	router := gin.New()
	router.Use(SessionMiddleware(store, "session", 30*time.Minute, false))
	router.GET("/test", func(c *gin.Context) {
		c.Status(http.StatusOK)
	})

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusUnauthorized, w.Code)
}

func TestSessionMiddleware_InvalidSession(t *testing.T) {
	store := newTestStore()
	router := gin.New()
	router.Use(SessionMiddleware(store, "session", 30*time.Minute, false))
	router.GET("/test", func(c *gin.Context) {
		c.Status(http.StatusOK)
	})

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	req.AddCookie(&http.Cookie{Name: "session", Value: "nonexistent"})
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusUnauthorized, w.Code)
}

func TestSessionMiddleware_ValidSession(t *testing.T) {
	store := newTestStore()
	store.sessions["valid-session"] = &session.Data{
		AccessToken: "token-123",
		CSRFToken:   "csrf-abc",
		// ExpiresAt を未来に設定して IsExpired() が false を返すようにする
		ExpiresAt: time.Now().Add(1 * time.Hour).Unix(),
	}

	router := gin.New()
	router.Use(SessionMiddleware(store, "session", 30*time.Minute, false))
	router.GET("/test", func(c *gin.Context) {
		sess, ok := GetSessionData(c)
		require.True(t, ok)
		assert.Equal(t, "token-123", sess.AccessToken)

		id, ok := GetSessionID(c)
		require.True(t, ok)
		assert.Equal(t, "valid-session", id)

		c.Status(http.StatusOK)
	})

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	req.AddCookie(&http.Cookie{Name: "session", Value: "valid-session"})
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)
}

// TestSessionMiddleware_ExpiredSession はアクセストークン期限切れセッションが 401 を返すことを確認する。
// Finding 11: ExpiresAt が過去の場合はミドルウェアで拒否する（Redis TTL のみに依存しない）。
func TestSessionMiddleware_ExpiredSession(t *testing.T) {
	store := newTestStore()
	store.sessions["expired-session"] = &session.Data{
		AccessToken: "expired-token",
		ExpiresAt:   time.Now().Add(-1 * time.Hour).Unix(),
	}

	router := gin.New()
	router.Use(SessionMiddleware(store, "session", 30*time.Minute, false))
	router.GET("/test", func(c *gin.Context) {
		c.Status(http.StatusOK)
	})

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	req.AddCookie(&http.Cookie{Name: "session", Value: "expired-session"})
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusUnauthorized, w.Code)
}

// TestSessionMiddleware_ExpiredSessionWithRefreshToken は expired + refresh token ありの場合に
// ハンドラーが呼ばれ session_needs_refresh フラグが設定されることを確認する。
func TestSessionMiddleware_ExpiredSessionWithRefreshToken(t *testing.T) {
	store := newTestStore()
	store.sessions["expired-refresh-session"] = &session.Data{
		AccessToken:  "expired-token",
		RefreshToken: "refresh-token-123",
		ExpiresAt:    time.Now().Add(-1 * time.Hour).Unix(),
	}

	var handlerCalled bool
	var needsRefresh bool

	router := gin.New()
	router.Use(SessionMiddleware(store, "session", 30*time.Minute, false))
	router.GET("/test", func(c *gin.Context) {
		handlerCalled = true
		val, exists := c.Get(SessionNeedsRefreshKey)
		if exists {
			if b, ok := val.(bool); ok {
				needsRefresh = b
			}
		}
		c.Status(http.StatusOK)
	})

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	req.AddCookie(&http.Cookie{Name: "session", Value: "expired-refresh-session"})
	router.ServeHTTP(w, req)

	// handler が呼ばれること（401 で止まらないこと）
	assert.Equal(t, http.StatusOK, w.Code)
	assert.True(t, handlerCalled, "handler should be called when refresh token is present")
	assert.True(t, needsRefresh, "session_needs_refresh flag should be set")
}

func TestSessionMiddleware_SlidingTTL(t *testing.T) {
	store := newTestStore()
	store.sessions["sliding-session"] = &session.Data{
		AccessToken: "token",
		// ExpiresAt を未来に設定して IsExpired() が false を返すようにする
		ExpiresAt: time.Now().Add(1 * time.Hour).Unix(),
	}

	ttl := 30 * time.Minute
	router := gin.New()
	router.Use(SessionMiddleware(store, "session", ttl, true))
	router.GET("/test", func(c *gin.Context) {
		c.Status(http.StatusOK)
	})

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	req.AddCookie(&http.Cookie{Name: "session", Value: "sliding-session"})
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)
	assert.Equal(t, ttl, store.touched["sliding-session"])
}
