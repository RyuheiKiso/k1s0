package k1s0observability

import (
	"context"
	"encoding/json"
	"errors"
	"strings"
	"testing"

	"go.uber.org/zap"
)

func TestConfigBuilderSuccess(t *testing.T) {
	config, err := NewConfigBuilder().
		ServiceName("test-service").
		Env("dev").
		Version("1.0.0").
		Build()

	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if config.ServiceName != "test-service" {
		t.Errorf("expected 'test-service', got '%s'", config.ServiceName)
	}
	if config.Env != "dev" {
		t.Errorf("expected 'dev', got '%s'", config.Env)
	}
	if config.Version != "1.0.0" {
		t.Errorf("expected '1.0.0', got '%s'", config.Version)
	}
}

func TestConfigBuilderMissingServiceName(t *testing.T) {
	_, err := NewConfigBuilder().
		Env("dev").
		Build()

	if err == nil {
		t.Fatal("expected error for missing service name")
	}
	if !errors.Is(err, ErrMissingServiceName) {
		t.Errorf("expected ErrMissingServiceName, got %v", err)
	}
}

func TestConfigBuilderMissingEnv(t *testing.T) {
	_, err := NewConfigBuilder().
		ServiceName("test").
		Build()

	if err == nil {
		t.Fatal("expected error for missing env")
	}
	if !errors.Is(err, ErrMissingEnv) {
		t.Errorf("expected ErrMissingEnv, got %v", err)
	}
}

func TestConfigBuilderInvalidEnv(t *testing.T) {
	_, err := NewConfigBuilder().
		ServiceName("test").
		Env("invalid").
		Build()

	if err == nil {
		t.Fatal("expected error for invalid env")
	}
}

func TestConfigDefaults(t *testing.T) {
	config, err := NewConfigBuilder().
		ServiceName("test").
		Env("dev").
		Build()

	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if config.LogLevel != "INFO" {
		t.Errorf("expected 'INFO', got '%s'", config.LogLevel)
	}
	if config.SamplingRate != 1.0 {
		t.Errorf("expected 1.0, got %f", config.SamplingRate)
	}
}

func TestConfigSamplingRateClamped(t *testing.T) {
	config, _ := NewConfigBuilder().
		ServiceName("test").
		Env("dev").
		SamplingRate(1.5).
		Build()

	if config.SamplingRate != 1.0 {
		t.Errorf("expected 1.0, got %f", config.SamplingRate)
	}

	config, _ = NewConfigBuilder().
		ServiceName("test").
		Env("dev").
		SamplingRate(-0.5).
		Build()

	if config.SamplingRate != 0.0 {
		t.Errorf("expected 0.0, got %f", config.SamplingRate)
	}
}

func TestConfigIsProduction(t *testing.T) {
	devConfig, _ := NewConfigBuilder().
		ServiceName("test").
		Env("dev").
		Build()
	if devConfig.IsProduction() {
		t.Error("dev should not be production")
	}

	prodConfig, _ := NewConfigBuilder().
		ServiceName("test").
		Env("prod").
		Build()
	if !prodConfig.IsProduction() {
		t.Error("prod should be production")
	}
}

func TestRequestContext(t *testing.T) {
	ctx := NewRequestContext()

	if ctx.TraceID == "" {
		t.Error("expected trace ID to be generated")
	}
	if ctx.RequestID == "" {
		t.Error("expected request ID to be generated")
	}
}

func TestRequestContextWithTraceID(t *testing.T) {
	ctx := NewRequestContextWithTraceID("custom-trace")

	if ctx.TraceID != "custom-trace" {
		t.Errorf("expected 'custom-trace', got '%s'", ctx.TraceID)
	}
	if ctx.RequestID == "" {
		t.Error("expected request ID to be generated")
	}
}

func TestRequestContextMethods(t *testing.T) {
	ctx := NewRequestContext().
		WithUserID("user-123").
		WithTenantID("tenant-456").
		WithSpanID("span-789").
		WithExtra("key", "value")

	if ctx.UserID != "user-123" {
		t.Errorf("expected 'user-123', got '%s'", ctx.UserID)
	}
	if ctx.TenantID != "tenant-456" {
		t.Errorf("expected 'tenant-456', got '%s'", ctx.TenantID)
	}
	if ctx.SpanID != "span-789" {
		t.Errorf("expected 'span-789', got '%s'", ctx.SpanID)
	}
	if ctx.Extra["key"] != "value" {
		t.Errorf("expected 'value', got '%s'", ctx.Extra["key"])
	}
}

func TestRequestContextToFromContext(t *testing.T) {
	rc := NewRequestContext().WithUserID("user-123")
	ctx := rc.ToContext(context.Background())

	retrieved := FromContext(ctx)
	if retrieved == nil {
		t.Fatal("expected to retrieve request context")
	}
	if retrieved.UserID != "user-123" {
		t.Errorf("expected 'user-123', got '%s'", retrieved.UserID)
	}
}

func TestFromContextOrNew(t *testing.T) {
	// With existing context
	rc := NewRequestContext().WithUserID("existing")
	ctx := rc.ToContext(context.Background())

	retrieved := FromContextOrNew(ctx)
	if retrieved.UserID != "existing" {
		t.Errorf("expected 'existing', got '%s'", retrieved.UserID)
	}

	// Without existing context
	newCtx := FromContextOrNew(context.Background())
	if newCtx == nil {
		t.Fatal("expected new context to be created")
	}
	if newCtx.TraceID == "" {
		t.Error("expected trace ID to be generated")
	}
}

func TestLogEntry(t *testing.T) {
	config, _ := NewConfigBuilder().
		ServiceName("test-service").
		Env("dev").
		Build()

	ctx := NewRequestContext()
	entry := Info("test message").
		WithContext(ctx).
		WithService(config)

	jsonBytes, err := entry.ToJSON()
	if err != nil {
		t.Fatalf("failed to marshal: %v", err)
	}

	jsonStr := string(jsonBytes)
	if !strings.Contains(jsonStr, "test-service") {
		t.Error("expected service name in JSON")
	}
	if !strings.Contains(jsonStr, "test message") {
		t.Error("expected message in JSON")
	}
	if !strings.Contains(jsonStr, "INFO") {
		t.Error("expected level in JSON")
	}
}

func TestLogEntryLevels(t *testing.T) {
	tests := []struct {
		entry    *LogEntry
		expected LogLevel
	}{
		{Debug("msg"), LevelDebug},
		{Info("msg"), LevelInfo},
		{Warn("msg"), LevelWarn},
		{Error("msg"), LevelError},
	}

	for _, tt := range tests {
		if tt.entry.Level != tt.expected {
			t.Errorf("expected %s, got %s", tt.expected, tt.entry.Level)
		}
	}
}

func TestLogEntryWithField(t *testing.T) {
	entry := Info("test").WithField("custom_key", "custom_value")

	jsonBytes, err := entry.ToJSON()
	if err != nil {
		t.Fatalf("failed to marshal: %v", err)
	}

	var result map[string]any
	if err := json.Unmarshal(jsonBytes, &result); err != nil {
		t.Fatalf("failed to unmarshal: %v", err)
	}

	if result["custom_key"] != "custom_value" {
		t.Errorf("expected 'custom_value', got '%v'", result["custom_key"])
	}
}

func TestLogEntryString(t *testing.T) {
	entry := Info("test message")
	str := entry.String()

	if str == "" {
		t.Error("expected non-empty string")
	}
	if !strings.Contains(str, "test message") {
		t.Error("expected message in string")
	}
}

func TestNewLogger(t *testing.T) {
	config, err := NewConfigBuilder().
		ServiceName("test-service").
		Env("dev").
		Build()
	if err != nil {
		t.Fatalf("failed to create config: %v", err)
	}

	logger, err := NewLogger(config)
	if err != nil {
		t.Fatalf("failed to create logger: %v", err)
	}

	if logger == nil {
		t.Fatal("expected logger to be created")
	}
	if logger.Config() != config {
		t.Error("expected config to match")
	}
}

func TestLoggerWithContext(t *testing.T) {
	config, _ := NewConfigBuilder().
		ServiceName("test-service").
		Env("dev").
		Build()

	logger, _ := NewLogger(config)

	rc := NewRequestContext().
		WithUserID("user-123").
		WithTenantID("tenant-456")
	ctx := rc.ToContext(context.Background())

	// This should not panic
	logger.Info(ctx, "test message", zap.String("extra", "value"))
}

func TestLoggerWithFields(t *testing.T) {
	config, _ := NewConfigBuilder().
		ServiceName("test-service").
		Env("dev").
		Build()

	logger, _ := NewLogger(config)

	newLogger := logger.WithFields(zap.String("component", "auth"))
	if newLogger == logger {
		t.Error("expected new logger instance")
	}
}

func TestLoggerWithError(t *testing.T) {
	config, _ := NewConfigBuilder().
		ServiceName("test-service").
		Env("dev").
		Build()

	logger, _ := NewLogger(config)
	err := errors.New("test error")

	newLogger := logger.WithError(err)
	if newLogger == logger {
		t.Error("expected new logger instance")
	}
}

func TestIsValidEnv(t *testing.T) {
	validEnvs := []string{"dev", "stg", "prod"}
	for _, env := range validEnvs {
		if !IsValidEnv(env) {
			t.Errorf("expected %s to be valid", env)
		}
	}

	invalidEnvs := []string{"default", "production", ""}
	for _, env := range invalidEnvs {
		if IsValidEnv(env) {
			t.Errorf("expected %s to be invalid", env)
		}
	}
}

func TestConfigNewRequestContext(t *testing.T) {
	config, _ := NewConfigBuilder().
		ServiceName("test").
		Env("dev").
		Build()

	ctx := config.NewRequestContext()
	if ctx == nil {
		t.Fatal("expected context to be created")
	}
	if ctx.TraceID == "" {
		t.Error("expected trace ID")
	}
}

func TestConfigNewRequestContextWithTrace(t *testing.T) {
	config, _ := NewConfigBuilder().
		ServiceName("test").
		Env("dev").
		Build()

	ctx := config.NewRequestContextWithTrace("custom-trace")
	if ctx.TraceID != "custom-trace" {
		t.Errorf("expected 'custom-trace', got '%s'", ctx.TraceID)
	}
}

func TestFromContextNil(t *testing.T) {
	// Test with nil context
	rc := FromContext(nil)
	if rc != nil {
		t.Error("expected nil for nil context")
	}

	// Test with context without request context
	rc = FromContext(context.Background())
	if rc != nil {
		t.Error("expected nil for context without request context")
	}
}
