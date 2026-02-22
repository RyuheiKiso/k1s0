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
	sessions map[string]*session.SessionData
	touched  map[string]time.Duration
}

func newTestStore() *testStore {
	return &testStore{
		sessions: make(map[string]*session.SessionData),
		touched:  make(map[string]time.Duration),
	}
}

func (s *testStore) Create(_ context.Context, data *session.SessionData, _ time.Duration) (string, error) {
	id := "test-id"
	s.sessions[id] = data
	return id, nil
}

func (s *testStore) Get(_ context.Context, id string) (*session.SessionData, error) {
	d, ok := s.sessions[id]
	if !ok {
		return nil, nil
	}
	return d, nil
}

func (s *testStore) Update(_ context.Context, id string, data *session.SessionData, _ time.Duration) error {
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
	store.sessions["valid-session"] = &session.SessionData{
		AccessToken: "token-123",
		CSRFToken:   "csrf-abc",
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

func TestSessionMiddleware_SlidingTTL(t *testing.T) {
	store := newTestStore()
	store.sessions["sliding-session"] = &session.SessionData{AccessToken: "token"}

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
