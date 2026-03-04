package middleware

import "github.com/gin-gonic/gin"

// GetRequestID returns the correlation ID from context/header as request_id.
func GetRequestID(c *gin.Context) string {
	if v, ok := c.Get(CorrelationIDKey); ok {
		if id, ok := v.(string); ok && id != "" {
			return id
		}
	}
	if id := c.GetHeader(HeaderCorrelationID); id != "" {
		return id
	}
	return ""
}
