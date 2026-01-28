package k1s0grpcserver

import (
	"context"
	"testing"
	"time"

	k1s0obs "github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-observability"
	"github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-grpc-server/interceptors"
)

func TestNewConfig(t *testing.T) {
	config := NewConfig()

	if config.Port != 50051 {
		t.Errorf("expected port 50051, got %d", config.Port)
	}
	if config.Host != "0.0.0.0" {
		t.Errorf("expected host '0.0.0.0', got '%s'", config.Host)
	}
	if !config.EnableRecovery {
		t.Error("expected recovery to be enabled by default")
	}
	if !config.EnableLogging {
		t.Error("expected logging to be enabled by default")
	}
}

func TestConfigAddress(t *testing.T) {
	config := NewConfig().WithHost("localhost").WithPort(8080)
	if config.Address() != "localhost:8080" {
		t.Errorf("expected 'localhost:8080', got '%s'", config.Address())
	}
}

func TestConfigChaining(t *testing.T) {
	config := NewConfig().
		WithPort(9000).
		WithHost("127.0.0.1").
		WithServiceName("test-service").
		WithReflection(true).
		WithHealthCheck(true).
		WithMaxRecvMsgSize(8 * 1024 * 1024).
		WithMaxSendMsgSize(8 * 1024 * 1024).
		WithMaxConcurrentStreams(500).
		WithConnectionTimeout(60 * time.Second).
		WithDefaultDeadline(15 * time.Second).
		WithRecovery(true).
		WithLogging(true).
		WithTracing(true).
		WithDeadline(true)

	if config.Port != 9000 {
		t.Errorf("expected port 9000, got %d", config.Port)
	}
	if config.Host != "127.0.0.1" {
		t.Errorf("expected host '127.0.0.1', got '%s'", config.Host)
	}
	if config.ServiceName != "test-service" {
		t.Errorf("expected service name 'test-service', got '%s'", config.ServiceName)
	}
	if !config.EnableReflection {
		t.Error("expected reflection to be enabled")
	}
	if config.MaxRecvMsgSize != 8*1024*1024 {
		t.Errorf("expected MaxRecvMsgSize 8MB, got %d", config.MaxRecvMsgSize)
	}
	if config.DefaultDeadline != 15*time.Second {
		t.Errorf("expected DefaultDeadline 15s, got %v", config.DefaultDeadline)
	}
}

func TestNewServer(t *testing.T) {
	config := NewConfig().
		WithServiceName("test-service").
		WithPort(0) // Use any available port

	obsConfig, err := k1s0obs.NewConfigBuilder().
		ServiceName("test-service").
		Env("dev").
		Build()
	if err != nil {
		t.Fatalf("failed to create obs config: %v", err)
	}

	server, err := NewServer(config, obsConfig)
	if err != nil {
		t.Fatalf("failed to create server: %v", err)
	}

	if server.GRPCServer() == nil {
		t.Error("expected gRPC server to be created")
	}
	if server.Logger() == nil {
		t.Error("expected logger to be created")
	}
	if server.HealthServer() == nil {
		t.Error("expected health server to be created")
	}
}

func TestServerWithoutHealthCheck(t *testing.T) {
	config := NewConfig().
		WithServiceName("test-service").
		WithHealthCheck(false)

	obsConfig, _ := k1s0obs.NewConfigBuilder().
		ServiceName("test-service").
		Env("dev").
		Build()

	server, err := NewServer(config, obsConfig)
	if err != nil {
		t.Fatalf("failed to create server: %v", err)
	}

	if server.HealthServer() != nil {
		t.Error("expected health server to be nil when disabled")
	}
}

func TestTracingInterceptor(t *testing.T) {
	// Test that the interceptor can be created
	interceptor := interceptors.TracingInterceptor()
	if interceptor == nil {
		t.Error("expected interceptor to be created")
	}
}

func TestDeadlineInterceptor(t *testing.T) {
	interceptor := interceptors.DeadlineInterceptor(5 * time.Second)
	if interceptor == nil {
		t.Error("expected interceptor to be created")
	}
}

func TestErrorInterceptor(t *testing.T) {
	interceptor := interceptors.ErrorInterceptor()
	if interceptor == nil {
		t.Error("expected interceptor to be created")
	}
}

func TestDeadlineFromContext(t *testing.T) {
	// Context without deadline
	ctx := context.Background()
	if d := interceptors.DeadlineFromContext(ctx); d != 0 {
		t.Errorf("expected 0 for no deadline, got %v", d)
	}

	// Context with deadline
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	d := interceptors.DeadlineFromContext(ctx)
	if d <= 0 || d > 5*time.Second {
		t.Errorf("expected deadline between 0 and 5s, got %v", d)
	}
}

func TestHasDeadline(t *testing.T) {
	ctx := context.Background()
	if interceptors.HasDeadline(ctx) {
		t.Error("expected no deadline for background context")
	}

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	if !interceptors.HasDeadline(ctx) {
		t.Error("expected deadline for context with timeout")
	}
}

func TestInjectTraceMetadata(t *testing.T) {
	reqCtx := k1s0obs.NewRequestContext().
		WithSpanID("span-123")

	ctx := reqCtx.ToContext(context.Background())
	ctx = interceptors.InjectTraceMetadata(ctx)

	// The context should have outgoing metadata
	// (We can't easily verify the metadata contents without more setup)
	if ctx == nil {
		t.Error("expected context to be non-nil")
	}
}

func TestItoa(t *testing.T) {
	tests := []struct {
		input    int
		expected string
	}{
		{0, "0"},
		{1, "1"},
		{42, "42"},
		{12345, "12345"},
		{-1, "-1"},
		{-42, "-42"},
	}

	for _, tt := range tests {
		t.Run(tt.expected, func(t *testing.T) {
			result := itoa(tt.input)
			if result != tt.expected {
				t.Errorf("expected %s, got %s", tt.expected, result)
			}
		})
	}
}
