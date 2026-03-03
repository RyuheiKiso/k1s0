package handler

import (
	"net/http"

	"github.com/gin-gonic/gin"
	"github.com/redis/go-redis/v9"
)

// HealthHandler provides liveness and readiness probes.
type HealthHandler struct {
	redisClient redis.Cmdable
}

// NewHealthHandler creates a new HealthHandler.
func NewHealthHandler(redisClient redis.Cmdable) *HealthHandler {
	return &HealthHandler{redisClient: redisClient}
}

// Healthz is the liveness probe. Returns 200 if the process is alive.
func (h *HealthHandler) Healthz(c *gin.Context) {
	c.JSON(http.StatusOK, gin.H{"status": "ok"})
}

// Readyz is the readiness probe. Checks Redis connectivity.
func (h *HealthHandler) Readyz(c *gin.Context) {
	if err := h.redisClient.Ping(c.Request.Context()).Err(); err != nil {
		c.JSON(http.StatusServiceUnavailable, gin.H{
			"status": "not_ready",
			"reason": "redis connection failed",
		})
		return
	}
	c.JSON(http.StatusOK, gin.H{"status": "ready"})
}
