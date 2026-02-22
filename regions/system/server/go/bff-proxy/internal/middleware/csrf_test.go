package middleware

import (
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"

	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/session"
)

func TestCSRFMiddleware_SafeMethodsExempt(t *testing.T) {
	store := newTestStore()
	router := gin.New()
	router.Use(CSRFMiddleware(store, "", "session"))
	router.GET("/test", func(c *gin.Context) {
		c.Status(http.StatusOK)
	})

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)
}

func TestCSRFMiddleware_NoSession(t *testing.T) {
	store := newTestStore()
	router := gin.New()
	router.Use(CSRFMiddleware(store, "", "session"))
	router.POST("/test", func(c *gin.Context) {
		c.Status(http.StatusOK)
	})

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodPost, "/test", nil)
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusForbidden, w.Code)
}

func TestCSRFMiddleware_MismatchToken(t *testing.T) {
	store := newTestStore()
	store.sessions["my-session"] = &session.SessionData{
		AccessToken: "token",
		CSRFToken:   "correct-token",
		ExpiresAt:   time.Now().Add(10 * time.Minute).Unix(),
	}

	router := gin.New()
	router.Use(CSRFMiddleware(store, "X-CSRF-Token", "session"))
	router.POST("/test", func(c *gin.Context) {
		c.Status(http.StatusOK)
	})

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodPost, "/test", nil)
	req.AddCookie(&http.Cookie{Name: "session", Value: "my-session"})
	req.Header.Set("X-CSRF-Token", "wrong-token")
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusForbidden, w.Code)
}

func TestCSRFMiddleware_ValidToken(t *testing.T) {
	store := newTestStore()
	store.sessions["my-session"] = &session.SessionData{
		AccessToken: "token",
		CSRFToken:   "valid-csrf",
		ExpiresAt:   time.Now().Add(10 * time.Minute).Unix(),
	}

	router := gin.New()
	router.Use(CSRFMiddleware(store, "X-CSRF-Token", "session"))
	router.POST("/test", func(c *gin.Context) {
		c.Status(http.StatusOK)
	})

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodPost, "/test", nil)
	req.AddCookie(&http.Cookie{Name: "session", Value: "my-session"})
	req.Header.Set("X-CSRF-Token", "valid-csrf")
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)
}
