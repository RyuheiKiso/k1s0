package middleware

import (
	"log/slog"

	"github.com/gin-gonic/gin"
	"go.opentelemetry.io/otel/trace"
)

// OTelTraceIDMiddleware は OpenTelemetry のトレースIDをリクエストコンテキストに設定する。
// slog.SetDefault はグローバル状態を変更するため並行リクエストでデータ競合を起こす。
// リクエストスコープのロガーを Gin コンテキストに格納して回避する。
func OTelTraceIDMiddleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		spanCtx := trace.SpanContextFromContext(c.Request.Context())
		if spanCtx.HasTraceID() {
			traceID := spanCtx.TraceID().String()
			c.Set(TraceIDKey, traceID)
			c.Header(HeaderTraceID, traceID)

			// リクエストスコープのロガーをコンテキストに格納（グローバル変更を回避）
			logger := slog.Default().With(slog.String("trace_id", traceID))
			c.Set("logger", logger)
		}

		c.Next()
	}
}
