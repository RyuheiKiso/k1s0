package middleware

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

func TestCorrelationMiddleware_GeneratesIDs(t *testing.T) {
	router := gin.New()
	router.Use(CorrelationMiddleware())
	router.GET("/test", func(c *gin.Context) {
		cid, _ := c.Get(CorrelationIDKey)
		tid, _ := c.Get(TraceIDKey)
		assert.NotEmpty(t, cid)
		assert.NotEmpty(t, tid)
		c.Status(http.StatusOK)
	})

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)
	assert.NotEmpty(t, w.Header().Get(HeaderCorrelationID))
	assert.NotEmpty(t, w.Header().Get(HeaderTraceID))
}

func TestCorrelationMiddleware_PropagatesExisting(t *testing.T) {
	router := gin.New()
	router.Use(CorrelationMiddleware())
	router.GET("/test", func(c *gin.Context) {
		cid, _ := c.Get(CorrelationIDKey)
		assert.Equal(t, "existing-correlation-id", cid)
		c.Status(http.StatusOK)
	})

	w := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	req.Header.Set(HeaderCorrelationID, "existing-correlation-id")
	router.ServeHTTP(w, req)

	assert.Equal(t, http.StatusOK, w.Code)
	assert.Equal(t, "existing-correlation-id", w.Header().Get(HeaderCorrelationID))
}

func TestGenerateTraceID_Format(t *testing.T) {
	id := generateTraceID()
	assert.Len(t, id, 32, "trace ID should be 32 hex characters")

	for _, c := range id {
		assert.True(t, (c >= '0' && c <= '9') || (c >= 'a' && c <= 'f'),
			"trace ID should only contain lowercase hex characters, got: %c", c)
	}
}
