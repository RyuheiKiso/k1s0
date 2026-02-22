package middleware

import (
	"log/slog"

	"github.com/gin-gonic/gin"
	"go.opentelemetry.io/otel/trace"
)

// OTelTraceIDMiddleware extracts the OpenTelemetry trace ID from the current
// span context and sets it on the Gin context key and response header.
// It also enriches the default slog logger with the trace_id field.
func OTelTraceIDMiddleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		spanCtx := trace.SpanContextFromContext(c.Request.Context())
		if spanCtx.HasTraceID() {
			traceID := spanCtx.TraceID().String()
			c.Set(TraceIDKey, traceID)
			c.Header(HeaderTraceID, traceID)

			// Enrich slog with trace_id for downstream handlers.
			logger := slog.Default().With(slog.String("trace_id", traceID))
			slog.SetDefault(logger)
		}

		c.Next()
	}
}
