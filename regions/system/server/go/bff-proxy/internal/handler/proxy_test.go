package handler

import (
	"context"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/middleware"
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
		w.Write([]byte(`{"result": "ok"}`))
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
