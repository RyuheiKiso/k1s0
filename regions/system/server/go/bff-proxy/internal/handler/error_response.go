package handler

import (
	"net/http"

	"github.com/gin-gonic/gin"

	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/middleware"
)

func respondError(c *gin.Context, status int, code string) {
	payload := gin.H{
		"error":      code,
		"request_id": middleware.GetRequestID(c),
	}
	c.JSON(status, payload)
}

func respondErrorWithMessage(c *gin.Context, status int, code, message string) {
	payload := gin.H{
		"error":      code,
		"message":    message,
		"request_id": middleware.GetRequestID(c),
	}
	c.JSON(status, payload)
}

func respondBadRequest(c *gin.Context, code string) {
	respondError(c, http.StatusBadRequest, code)
}

func abortErrorWithMessage(c *gin.Context, status int, code, message string) {
	payload := gin.H{
		"error":      code,
		"message":    message,
		"request_id": middleware.GetRequestID(c),
	}
	c.AbortWithStatusJSON(status, payload)
}
