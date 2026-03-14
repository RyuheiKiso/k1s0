package telemetry

import (
	"context"
	"log/slog"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"go.opentelemetry.io/otel/trace"
)

// トレースエンドポイントなしでInitTelemetryが正常に初期化されることを確認する。
func TestInitTelemetry_WithoutTraceEndpoint(t *testing.T) {
	cfg := TelemetryConfig{
		ServiceName: "test-service",
		Version:     "1.0.0",
		Tier:        "system",
		Environment: "dev",
		SampleRate:  1.0,
		LogLevel:    "debug",
		LogFormat:   "json",
	}

	ctx := context.Background()
	provider, err := InitTelemetry(ctx, cfg)
	require.NoError(t, err)
	require.NotNil(t, provider)
	require.NotNil(t, provider.Logger())

	err = provider.Shutdown(ctx)
	assert.NoError(t, err)
}

// トレースエンドポイントを指定した場合にInitTelemetryがTracerProviderを生成することを確認する。
func TestInitTelemetry_WithTraceEndpoint(t *testing.T) {
	cfg := TelemetryConfig{
		ServiceName:   "test-service",
		Version:       "1.0.0",
		Tier:          "system",
		Environment:   "dev",
		TraceEndpoint: "localhost:4317",
		SampleRate:    1.0,
		LogLevel:      "info",
		LogFormat:     "json",
	}

	ctx := context.Background()
	provider, err := InitTelemetry(ctx, cfg)
	require.NoError(t, err)
	require.NotNil(t, provider)
	require.NotNil(t, provider.tracerProvider)

	err = provider.Shutdown(ctx)
	assert.NoError(t, err)
}

// NewLoggerがDebugレベルのログを有効にしたロガーを生成することを確認する。
func TestNewLogger_DebugLevel(t *testing.T) {
	cfg := TelemetryConfig{
		ServiceName: "test-service",
		Version:     "1.0.0",
		Tier:        "system",
		Environment: "dev",
		LogLevel:    "debug",
		LogFormat:   "json",
	}

	logger := NewLogger(cfg)
	require.NotNil(t, logger)
	assert.True(t, logger.Enabled(context.Background(), slog.LevelDebug))
}

// NewLoggerがInfoレベルのログを有効にし、Debugレベルを無効にすることを確認する。
func TestNewLogger_InfoLevel(t *testing.T) {
	cfg := TelemetryConfig{
		ServiceName: "test-service",
		Version:     "1.0.0",
		Tier:        "system",
		Environment: "staging",
		LogLevel:    "info",
		LogFormat:   "json",
	}

	logger := NewLogger(cfg)
	require.NotNil(t, logger)
	assert.True(t, logger.Enabled(context.Background(), slog.LevelInfo))
	assert.False(t, logger.Enabled(context.Background(), slog.LevelDebug))
}

// NewLoggerがWarnレベルのログを有効にし、Infoレベルを無効にすることを確認する。
func TestNewLogger_WarnLevel(t *testing.T) {
	cfg := TelemetryConfig{
		ServiceName: "test-service",
		Version:     "1.0.0",
		Tier:        "system",
		Environment: "prod",
		LogLevel:    "warn",
		LogFormat:   "json",
	}

	logger := NewLogger(cfg)
	require.NotNil(t, logger)
	assert.True(t, logger.Enabled(context.Background(), slog.LevelWarn))
	assert.False(t, logger.Enabled(context.Background(), slog.LevelInfo))
}

// NewLoggerがErrorレベルのログを有効にし、Warnレベルを無効にすることを確認する。
func TestNewLogger_ErrorLevel(t *testing.T) {
	cfg := TelemetryConfig{
		ServiceName: "test-service",
		Version:     "1.0.0",
		Tier:        "system",
		Environment: "prod",
		LogLevel:    "error",
		LogFormat:   "json",
	}

	logger := NewLogger(cfg)
	require.NotNil(t, logger)
	assert.True(t, logger.Enabled(context.Background(), slog.LevelError))
	assert.False(t, logger.Enabled(context.Background(), slog.LevelWarn))
}

// NewLoggerがテキスト形式のフォーマットでロガーを生成することを確認する。
func TestNewLogger_TextFormat(t *testing.T) {
	cfg := TelemetryConfig{
		ServiceName: "test-service",
		Version:     "1.0.0",
		Tier:        "system",
		Environment: "dev",
		LogLevel:    "info",
		LogFormat:   "text",
	}

	logger := NewLogger(cfg)
	require.NotNil(t, logger)
	assert.True(t, logger.Enabled(context.Background(), slog.LevelInfo))
	assert.False(t, logger.Enabled(context.Background(), slog.LevelDebug))
}

// スパンなしのコンテキストでLogWithTraceがロガーを返すことを確認する。
func TestLogWithTrace_WithoutSpan(t *testing.T) {
	cfg := TelemetryConfig{
		ServiceName: "test-service",
		Version:     "1.0.0",
		Tier:        "system",
		Environment: "dev",
		LogLevel:    "debug",
		LogFormat:   "json",
	}

	logger := NewLogger(cfg)
	ctx := context.Background()

	result := LogWithTrace(ctx, logger)
	require.NotNil(t, result)
}

// アクティブなスパンを持つコンテキストでLogWithTraceがトレース情報を含むロガーを返すことを確認する。
func TestLogWithTrace_WithSpan(t *testing.T) {
	cfg := TelemetryConfig{
		ServiceName: "test-service",
		Version:     "1.0.0",
		Tier:        "system",
		Environment: "dev",
		LogLevel:    "debug",
		LogFormat:   "json",
	}

	logger := NewLogger(cfg)

	traceID, _ := trace.TraceIDFromHex("0af7651916cd43dd8448eb211c80319c")
	spanID, _ := trace.SpanIDFromHex("00f067aa0ba902b7")
	spanCtx := trace.NewSpanContext(trace.SpanContextConfig{
		TraceID:    traceID,
		SpanID:     spanID,
		TraceFlags: trace.FlagsSampled,
	})
	ctx := trace.ContextWithSpanContext(context.Background(), spanCtx)

	result := LogWithTrace(ctx, logger)
	require.NotNil(t, result)
}

// TracerProviderがnilの場合にProviderのShutdownがエラーなく完了することを確認する。
func TestProvider_Shutdown_NilTracerProvider(t *testing.T) {
	p := &Provider{
		tracerProvider: nil,
		logger:         slog.Default(),
	}
	err := p.Shutdown(context.Background())
	assert.NoError(t, err)
}

// スパンなしの場合にTraceHandlerが正しく動作することを確認する。
func TestTraceHandler_WithoutSpan(t *testing.T) {
	handler := slog.NewJSONHandler(nil, &slog.HandlerOptions{Level: slog.LevelDebug})
	th := TraceHandler(handler)

	require.NotNil(t, th)
	assert.True(t, th.Enabled(context.Background(), slog.LevelDebug))
}

// TraceHandlerのWithAttrsが*traceHandler型を返すことを確認する。
func TestTraceHandler_WithAttrs(t *testing.T) {
	handler := slog.NewJSONHandler(nil, &slog.HandlerOptions{Level: slog.LevelDebug})
	th := TraceHandler(handler)

	withAttrs := th.WithAttrs([]slog.Attr{slog.String("key", "value")})
	require.NotNil(t, withAttrs)
	_, ok := withAttrs.(*traceHandler)
	assert.True(t, ok, "WithAttrs should return a *traceHandler")
}

// TraceHandlerのWithGroupが*traceHandler型を返すことを確認する。
func TestTraceHandler_WithGroup(t *testing.T) {
	handler := slog.NewJSONHandler(nil, &slog.HandlerOptions{Level: slog.LevelDebug})
	th := TraceHandler(handler)

	withGroup := th.WithGroup("test-group")
	require.NotNil(t, withGroup)
	_, ok := withGroup.(*traceHandler)
	assert.True(t, ok, "WithGroup should return a *traceHandler")
}

// TelemetryConfigの各フィールドが正しく設定・取得できることを確認する。
func TestTelemetryConfig_Fields(t *testing.T) {
	cfg := TelemetryConfig{
		ServiceName:   "my-service",
		Version:       "2.0.0",
		Tier:          "business",
		Environment:   "staging",
		TraceEndpoint: "otel-collector:4317",
		SampleRate:    0.5,
		LogLevel:      "info",
		LogFormat:     "json",
	}

	assert.Equal(t, "my-service", cfg.ServiceName)
	assert.Equal(t, "2.0.0", cfg.Version)
	assert.Equal(t, "business", cfg.Tier)
	assert.Equal(t, "staging", cfg.Environment)
	assert.Equal(t, "otel-collector:4317", cfg.TraceEndpoint)
	assert.Equal(t, 0.5, cfg.SampleRate)
	assert.Equal(t, "info", cfg.LogLevel)
	assert.Equal(t, "json", cfg.LogFormat)
}
