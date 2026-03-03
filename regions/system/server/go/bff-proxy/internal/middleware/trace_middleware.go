package middleware

import (
	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
)

const (
	// HeaderCorrelationID is the HTTP header name for correlation IDs.
	HeaderCorrelationID = "X-Correlation-Id"

	// HeaderTraceID is the HTTP header name for trace IDs.
	HeaderTraceID = "X-Trace-Id"

	// CorrelationIDKey is the gin context key for the correlation ID.
	CorrelationIDKey = "correlation_id"

	// TraceIDKey is the gin context key for the trace ID.
	TraceIDKey = "trace_id"
)

// CorrelationMiddleware propagates or generates X-Correlation-Id and X-Trace-Id
// headers. Incoming values are reused; missing values are auto-generated.
func CorrelationMiddleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		correlationID := c.GetHeader(HeaderCorrelationID)
		if correlationID == "" {
			correlationID = uuid.New().String()
		}

		traceID := c.GetHeader(HeaderTraceID)
		if traceID == "" {
			traceID = generateTraceID()
		}

		c.Set(CorrelationIDKey, correlationID)
		c.Set(TraceIDKey, traceID)

		// Propagate to response headers.
		c.Header(HeaderCorrelationID, correlationID)
		c.Header(HeaderTraceID, traceID)

		c.Next()
	}
}

// generateTraceID produces a 32-character lowercase hex trace ID (UUID without hyphens).
func generateTraceID() string {
	id := uuid.New()
	raw := id[:]
	const hexChars = "0123456789abcdef"
	buf := make([]byte, 32)
	for i, b := range raw {
		buf[i*2] = hexChars[b>>4]
		buf[i*2+1] = hexChars[b&0x0f]
	}
	return string(buf)
}
