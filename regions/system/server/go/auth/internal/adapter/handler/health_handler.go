package handler

import (
	"context"
	"net/http"
	"time"

	"github.com/gin-gonic/gin"
)

// HealthChecker はサービスの健全性を確認するインターフェース。
type HealthChecker interface {
	Healthy(ctx context.Context) error
}

// HealthzHandler は GET /healthz のハンドラー。
func HealthzHandler() gin.HandlerFunc {
	return func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{
			"status": "ok",
		})
	}
}

// ReadyzHandler は GET /readyz のハンドラー。
// database と keycloak の接続確認を行う。
func ReadyzHandler(dbChecker HealthChecker, keycloakChecker HealthChecker) gin.HandlerFunc {
	return func(c *gin.Context) {
		ctx, cancel := context.WithTimeout(c.Request.Context(), 5*time.Second)
		defer cancel()

		checks := map[string]string{
			"database": "ok",
			"keycloak": "ok",
		}
		allReady := true

		if dbChecker != nil {
			if err := dbChecker.Healthy(ctx); err != nil {
				checks["database"] = "error: " + err.Error()
				allReady = false
			}
		}

		if keycloakChecker != nil {
			if err := keycloakChecker.Healthy(ctx); err != nil {
				checks["keycloak"] = "error: " + err.Error()
				allReady = false
			}
		}

		status := "ready"
		statusCode := http.StatusOK
		if !allReady {
			status = "not ready"
			statusCode = http.StatusServiceUnavailable
		}

		c.JSON(statusCode, gin.H{
			"status": status,
			"checks": checks,
		})
	}
}
