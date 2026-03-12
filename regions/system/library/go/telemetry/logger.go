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

	var handler slog.Handler
	if cfg.LogFormat == "text" {
		handler = slog.NewTextHandler(os.Stdout, &slog.HandlerOptions{Level: level})
	} else {
		handler = slog.NewJSONHandler(os.Stdout, &slog.HandlerOptions{Level: level})
	}
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

// traceHandler は slog.Handler をラップし、ログ出力時に OpenTelemetry の
// trace context から trace_id と span_id を自動的に注入する。
// LogWithTrace と異なり、呼び出し側で毎回コンテキストを渡す必要がない。
type traceHandler struct {
	next slog.Handler
}

// TraceHandler は slog.Handler をラップし、trace_id/span_id を自動注入するハンドラを返す。
// NewLogger で生成した Handler をラップして使用する。
//
//	handler := slog.NewJSONHandler(os.Stdout, opts)
//	logger := slog.New(TraceHandler(handler))
func TraceHandler(next slog.Handler) slog.Handler {
	return &traceHandler{next: next}
}

func (h *traceHandler) Enabled(ctx context.Context, level slog.Level) bool {
	return h.next.Enabled(ctx, level)
}

func (h *traceHandler) Handle(ctx context.Context, record slog.Record) error {
	spanCtx := trace.SpanContextFromContext(ctx)
	if spanCtx.HasTraceID() {
		record.AddAttrs(
			slog.String("trace_id", spanCtx.TraceID().String()),
			slog.String("span_id", spanCtx.SpanID().String()),
		)
	}
	return h.next.Handle(ctx, record)
}

func (h *traceHandler) WithAttrs(attrs []slog.Attr) slog.Handler {
	return &traceHandler{next: h.next.WithAttrs(attrs)}
}

func (h *traceHandler) WithGroup(name string) slog.Handler {
	return &traceHandler{next: h.next.WithGroup(name)}
}
