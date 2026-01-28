package k1s0grpcclient

import (
	"testing"
	"time"
)

// =============================================================================
// Config Tests
// =============================================================================

func TestDefaultClientConfig(t *testing.T) {
	config := DefaultClientConfig()

	if config.DefaultTimeout != 30*time.Second {
		t.Errorf("expected default timeout 30s, got %v", config.DefaultTimeout)
	}
	if config.KeepAlive.Time != 10*time.Second {
		t.Errorf("expected keep-alive time 10s, got %v", config.KeepAlive.Time)
	}
	if config.Retry.MaxAttempts != 3 {
		t.Errorf("expected max retry attempts 3, got %d", config.Retry.MaxAttempts)
	}
	if !config.Interceptors.Logging {
		t.Error("expected logging interceptor to be enabled")
	}
}

func TestClientConfig_Validate(t *testing.T) {
	config := &ClientConfig{
		DefaultTimeout: 0,
		KeepAlive: KeepAliveConfig{
			Time:    0,
			Timeout: 0,
		},
		Retry: RetryConfig{
			MaxAttempts: 0,
		},
	}

	validated := config.Validate()

	if validated.DefaultTimeout != 30*time.Second {
		t.Errorf("expected default timeout 30s, got %v", validated.DefaultTimeout)
	}
	if validated.KeepAlive.Time != 10*time.Second {
		t.Errorf("expected keep-alive time 10s, got %v", validated.KeepAlive.Time)
	}
	if validated.Retry.MaxAttempts != 3 {
		t.Errorf("expected max retry attempts 3, got %d", validated.Retry.MaxAttempts)
	}
}

func TestClientConfigBuilder(t *testing.T) {
	config := NewClientConfigBuilder().
		DefaultTimeout(10 * time.Second).
		KeepAliveTime(5 * time.Second).
		KeepAliveTimeout(2 * time.Second).
		MaxRetryAttempts(5).
		RetryBackoff(50*time.Millisecond, 5*time.Second, 1.5).
		EnableLogging(false).
		EnableTracing(true).
		EnableCircuitBreaker(true).
		UserAgent("test-client/1.0").
		MaxMessageSize(8 * 1024 * 1024).
		Build()

	if config.DefaultTimeout != 10*time.Second {
		t.Errorf("expected timeout 10s, got %v", config.DefaultTimeout)
	}
	if config.KeepAlive.Time != 5*time.Second {
		t.Errorf("expected keep-alive time 5s, got %v", config.KeepAlive.Time)
	}
	if config.Retry.MaxAttempts != 5 {
		t.Errorf("expected max attempts 5, got %d", config.Retry.MaxAttempts)
	}
	if config.Retry.BackoffMultiplier != 1.5 {
		t.Errorf("expected multiplier 1.5, got %f", config.Retry.BackoffMultiplier)
	}
	if config.Interceptors.Logging {
		t.Error("expected logging to be disabled")
	}
	if !config.Interceptors.CircuitBreaker {
		t.Error("expected circuit breaker to be enabled")
	}
	if config.UserAgent != "test-client/1.0" {
		t.Errorf("expected user agent 'test-client/1.0', got '%s'", config.UserAgent)
	}
	if config.MaxRecvMsgSize != 8*1024*1024 {
		t.Errorf("expected max recv size 8MB, got %d", config.MaxRecvMsgSize)
	}
}

// =============================================================================
// Pool Config Tests
// =============================================================================

func TestDefaultPoolConfig(t *testing.T) {
	config := DefaultPoolConfig()

	if config.MaxSize != 10 {
		t.Errorf("expected max size 10, got %d", config.MaxSize)
	}
	if config.MinSize != 2 {
		t.Errorf("expected min size 2, got %d", config.MinSize)
	}
	if config.MaxIdleTime != 30*time.Minute {
		t.Errorf("expected max idle time 30m, got %v", config.MaxIdleTime)
	}
	if config.WaitTimeout != 5*time.Second {
		t.Errorf("expected wait timeout 5s, got %v", config.WaitTimeout)
	}
}

// =============================================================================
// Backoff Calculation Tests
// =============================================================================

func TestCalculateBackoff(t *testing.T) {
	initial := 100 * time.Millisecond
	max := 10 * time.Second
	multiplier := 2.0

	tests := []struct {
		attempt  int
		expected time.Duration
	}{
		{1, 100 * time.Millisecond},
		{2, 200 * time.Millisecond},
		{3, 400 * time.Millisecond},
		{4, 800 * time.Millisecond},
	}

	for _, tt := range tests {
		result := calculateBackoff(tt.attempt, initial, max, multiplier)
		if result != tt.expected {
			t.Errorf("attempt %d: expected %v, got %v", tt.attempt, tt.expected, result)
		}
	}
}

func TestCalculateBackoff_MaxCap(t *testing.T) {
	initial := 1 * time.Second
	max := 5 * time.Second
	multiplier := 2.0

	// After several attempts, should be capped at max
	result := calculateBackoff(10, initial, max, multiplier)
	if result != max {
		t.Errorf("expected max %v, got %v", max, result)
	}
}

// =============================================================================
// Retryable Codes Tests
// =============================================================================

func TestParseRetryableCodes(t *testing.T) {
	strs := []string{"UNAVAILABLE", "RESOURCE_EXHAUSTED", "ABORTED", "INVALID"}
	codes := parseRetryableCodes(strs)

	// Should have 3 valid codes (INVALID is not a valid gRPC code)
	if len(codes) != 3 {
		t.Errorf("expected 3 codes, got %d", len(codes))
	}
}

// =============================================================================
// Pool Stats Tests
// =============================================================================

func TestPoolStats(t *testing.T) {
	stats := PoolStats{
		Size:      10,
		InUse:     3,
		Available: 5,
		Unhealthy: 2,
		Acquired:  100,
		Released:  97,
		Created:   15,
		Failed:    1,
	}

	if stats.Size != 10 {
		t.Errorf("expected size 10, got %d", stats.Size)
	}
	if stats.InUse != 3 {
		t.Errorf("expected in use 3, got %d", stats.InUse)
	}
	if stats.Available != 5 {
		t.Errorf("expected available 5, got %d", stats.Available)
	}
}
