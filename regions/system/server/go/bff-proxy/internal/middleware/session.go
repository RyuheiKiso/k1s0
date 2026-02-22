package middleware

import (
	"net/http"
	"time"

	"github.com/gin-gonic/gin"

	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/session"
)

const (
	// SessionDataKey is the gin context key where SessionData is stored.
	SessionDataKey = "bff_session"

	// SessionIDKey is the gin context key where the session ID is stored.
	SessionIDKey = "bff_session_id"
)

// SessionMiddleware extracts a session from the cookie, validates it,
// and stores it in the gin context. Optionally applies sliding TTL.
func SessionMiddleware(store session.Store, cookieName string, ttl time.Duration, sliding bool) gin.HandlerFunc {
	return func(c *gin.Context) {
		sessionID, err := c.Cookie(cookieName)
		if err != nil || sessionID == "" {
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{
				"error":   "BFF_SESSION_MISSING",
				"message": "Session cookie not found",
			})
			return
		}

		sess, err := store.Get(c.Request.Context(), sessionID)
		if err != nil || sess == nil {
			c.AbortWithStatusJSON(http.StatusUnauthorized, gin.H{
				"error":   "BFF_SESSION_INVALID",
				"message": "Session expired or invalid",
			})
			return
		}

		c.Set(SessionDataKey, sess)
		c.Set(SessionIDKey, sessionID)

		// Sliding window: extend TTL on each request.
		if sliding && ttl > 0 {
			_ = store.Touch(c.Request.Context(), sessionID, ttl)
		}

		c.Next()
	}
}

// GetSessionData retrieves SessionData from the gin context.
func GetSessionData(c *gin.Context) (*session.SessionData, bool) {
	val, exists := c.Get(SessionDataKey)
	if !exists {
		return nil, false
	}
	sess, ok := val.(*session.SessionData)
	return sess, ok
}

// GetSessionID retrieves the session ID from the gin context.
func GetSessionID(c *gin.Context) (string, bool) {
	val, exists := c.Get(SessionIDKey)
	if !exists {
		return "", false
	}
	id, ok := val.(string)
	return id, ok
}
