package middleware

import (
	"net/http"

	"github.com/gin-gonic/gin"

	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/session"
)

const (
	// DefaultCSRFHeader is the default header name for CSRF tokens.
	DefaultCSRFHeader = "X-CSRF-Token"
)

// CSRFMiddleware validates the CSRF token from the request header against the
// session-bound token. Only enforced for state-changing methods (POST, PUT, PATCH, DELETE).
func CSRFMiddleware(store session.Store, headerName string, sessionCookie string) gin.HandlerFunc {
	if headerName == "" {
		headerName = DefaultCSRFHeader
	}

	return func(c *gin.Context) {
		// Safe methods are exempt from CSRF checks.
		if c.Request.Method == http.MethodGet ||
			c.Request.Method == http.MethodHead ||
			c.Request.Method == http.MethodOptions {
			c.Next()
			return
		}

		sessionID, err := c.Cookie(sessionCookie)
		if err != nil || sessionID == "" {
			c.AbortWithStatusJSON(http.StatusForbidden, gin.H{
				"error":   "BFF_CSRF_NO_SESSION",
				"message": "Session not found",
			})
			return
		}

		sess, err := store.Get(c.Request.Context(), sessionID)
		if err != nil || sess == nil {
			c.AbortWithStatusJSON(http.StatusForbidden, gin.H{
				"error":   "BFF_CSRF_INVALID_SESSION",
				"message": "Invalid session",
			})
			return
		}

		csrfHeader := c.GetHeader(headerName)
		if csrfHeader == "" || csrfHeader != sess.CSRFToken {
			c.AbortWithStatusJSON(http.StatusForbidden, gin.H{
				"error":   "BFF_CSRF_MISMATCH",
				"message": "CSRF token mismatch",
			})
			return
		}

		c.Next()
	}
}
