package telemetry

import (
	"context"
	"log/slog"
	"os"

	"go.opentelemetry.io/otel/trace"
)

// NewLogger は TelemetryConfig に基づいて構造化ロガーを生成する。
// サービス名・バージョン・Tier・環境を標準フィールドとして付与する。
func NewLogger(cfg TelemetryConfig) *slog.Logger {
	level := slog.LevelWarn
	switch cfg.LogLevel {
	case "debug":
		level = slog.LevelDebug
	case "info":
		level = slog.LevelInfo
	case "warn":
		level = slog.LevelWarn
	case "error":
		level = slog.LevelError
	}

	handler := slog.NewJSONHandler(os.Stdout, &slog.HandlerOptions{Level: level})
	return slog.New(handler).With(
		slog.String("service", cfg.ServiceName),
		slog.String("version", cfg.Version),
		slog.String("tier", cfg.Tier),
		slog.String("environment", cfg.Environment),
	)
}

// LogWithTrace は OpenTelemetry のスパンコンテキストからトレース ID とスパン ID を
// ロガーに付与して返す。スパンが存在しない場合はそのまま返す。
func LogWithTrace(ctx context.Context, logger *slog.Logger) *slog.Logger {
	spanCtx := trace.SpanContextFromContext(ctx)
	if spanCtx.HasTraceID() {
		return logger.With(
			slog.String("trace_id", spanCtx.TraceID().String()),
			slog.String("span_id", spanCtx.SpanID().String()),
		)
	}
	return logger
}
