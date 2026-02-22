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

// mockRedisClient is a minimal mock for health check tests.
// Since HealthHandler only needs Ping(), we test via HTTP handlers directly.

func TestHealthz(t *testing.T) {
	handler := &HealthHandler{}

	router := gin.New()
	router.GET("/healthz", handler.Healthz)

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/healthz", nil)
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)
	assert.Contains(t, w.Body.String(), `"status":"ok"`)
}
