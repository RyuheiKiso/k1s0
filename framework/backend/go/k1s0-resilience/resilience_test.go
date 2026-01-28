package k1s0resilience

import (
	"context"
	"errors"
	"sync"
	"sync/atomic"
	"testing"
	"time"
)

// Test errors
var (
	errTest      = errors.New("test error")
	errRetryable = errors.New("retryable error")
)

// =============================================================================
// Retry Tests
// =============================================================================

func TestRetry_Success(t *testing.T) {
	ctx := context.Background()
	config := DefaultRetryConfig()

	attempts := 0
	result, err := Retry(ctx, config, func() (string, error) {
		attempts++
		return "success", nil
	})

	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	if result != "success" {
		t.Errorf("expected 'success', got %s", result)
	}
	if attempts != 1 {
		t.Errorf("expected 1 attempt, got %d", attempts)
	}
}

func TestRetry_SuccessAfterRetries(t *testing.T) {
	ctx := context.Background()
	config := DefaultRetryConfig()
	config.InitialInterval = time.Millisecond

	attempts := 0
	result, err := Retry(ctx, config, func() (string, error) {
		attempts++
		if attempts < 3 {
			return "", errTest
		}
		return "success", nil
	})

	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	if result != "success" {
		t.Errorf("expected 'success', got %s", result)
	}
	if attempts != 3 {
		t.Errorf("expected 3 attempts, got %d", attempts)
	}
}

func TestRetry_AllAttemptsFail(t *testing.T) {
	ctx := context.Background()
	config := DefaultRetryConfig()
	config.InitialInterval = time.Millisecond

	attempts := 0
	_, err := Retry(ctx, config, func() (string, error) {
		attempts++
		return "", errTest
	})

	if err == nil {
		t.Error("expected error, got nil")
	}
	if !IsRetryError(err) {
		t.Errorf("expected RetryError, got %T", err)
	}
	if GetRetryAttempts(err) != 3 {
		t.Errorf("expected 3 attempts, got %d", GetRetryAttempts(err))
	}
	if !errors.Is(err, errTest) {
		t.Error("expected error to wrap errTest")
	}
}

func TestRetry_RetryableChecker(t *testing.T) {
	ctx := context.Background()
	config := DefaultRetryConfig()
	config.InitialInterval = time.Millisecond
	config.RetryableChecker = func(err error) bool {
		return errors.Is(err, errRetryable)
	}

	// Non-retryable error should fail immediately
	attempts := 0
	_, err := Retry(ctx, config, func() (string, error) {
		attempts++
		return "", errTest
	})

	if err == nil {
		t.Error("expected error, got nil")
	}
	if attempts != 1 {
		t.Errorf("expected 1 attempt for non-retryable error, got %d", attempts)
	}
}

func TestRetry_ContextCancellation(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	config := DefaultRetryConfig()
	config.InitialInterval = 100 * time.Millisecond

	attempts := 0
	go func() {
		time.Sleep(50 * time.Millisecond)
		cancel()
	}()

	_, err := Retry(ctx, config, func() (string, error) {
		attempts++
		return "", errTest
	})

	if err == nil {
		t.Error("expected error, got nil")
	}
	// Should have attempted at least once
	if attempts < 1 {
		t.Errorf("expected at least 1 attempt, got %d", attempts)
	}
}

func TestRetryFunc(t *testing.T) {
	ctx := context.Background()
	config := DefaultRetryConfig()
	config.InitialInterval = time.Millisecond

	attempts := 0
	err := RetryFunc(ctx, config, func() error {
		attempts++
		if attempts < 2 {
			return errTest
		}
		return nil
	})

	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	if attempts != 2 {
		t.Errorf("expected 2 attempts, got %d", attempts)
	}
}

// =============================================================================
// Circuit Breaker Tests
// =============================================================================

func TestCircuitBreaker_ClosedState(t *testing.T) {
	config := DefaultCircuitBreakerConfig("test")
	cb := NewCircuitBreaker(config)

	result, err := cb.Execute(func() (interface{}, error) {
		return "success", nil
	})

	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	if result != "success" {
		t.Errorf("expected 'success', got %v", result)
	}
	if cb.State() != StateClosed {
		t.Errorf("expected closed state, got %s", cb.State())
	}
}

func TestCircuitBreaker_OpensAfterFailures(t *testing.T) {
	config := DefaultCircuitBreakerConfig("test")
	config.FailureThreshold = 3
	cb := NewCircuitBreaker(config)

	// Cause failures to open the circuit
	for i := 0; i < 3; i++ {
		_, _ = cb.Execute(func() (interface{}, error) {
			return nil, errTest
		})
	}

	if cb.State() != StateOpen {
		t.Errorf("expected open state, got %s", cb.State())
	}

	// Next call should fail immediately
	_, err := cb.Execute(func() (interface{}, error) {
		return "should not execute", nil
	})

	if !IsCircuitBreakerError(err) {
		t.Errorf("expected CircuitBreakerError, got %T", err)
	}
}

func TestCircuitBreaker_StateChange(t *testing.T) {
	stateChanges := make([]State, 0)
	config := DefaultCircuitBreakerConfig("test")
	config.FailureThreshold = 2
	config.Timeout = 50 * time.Millisecond
	config.OnStateChange = func(name string, from, to State) {
		stateChanges = append(stateChanges, to)
	}
	cb := NewCircuitBreaker(config)

	// Cause failures
	for i := 0; i < 2; i++ {
		_, _ = cb.Execute(func() (interface{}, error) {
			return nil, errTest
		})
	}

	// Wait for half-open transition
	time.Sleep(100 * time.Millisecond)

	// Trigger half-open state check
	_, _ = cb.Execute(func() (interface{}, error) {
		return "success", nil
	})

	if len(stateChanges) < 1 {
		t.Error("expected at least one state change")
	}
}

func TestCircuitBreaker_Counts(t *testing.T) {
	config := DefaultCircuitBreakerConfig("test")
	cb := NewCircuitBreaker(config)

	// Execute some successful calls
	for i := 0; i < 3; i++ {
		_, _ = cb.Execute(func() (interface{}, error) {
			return "success", nil
		})
	}

	// Execute a failing call
	_, _ = cb.Execute(func() (interface{}, error) {
		return nil, errTest
	})

	counts := cb.Counts()
	if counts.Requests != 4 {
		t.Errorf("expected 4 requests, got %d", counts.Requests)
	}
	if counts.TotalSuccesses != 3 {
		t.Errorf("expected 3 successes, got %d", counts.TotalSuccesses)
	}
	if counts.TotalFailures != 1 {
		t.Errorf("expected 1 failure, got %d", counts.TotalFailures)
	}
}

func TestCircuitBreakerGroup(t *testing.T) {
	group := NewCircuitBreakerGroup(func(name string) *CircuitBreakerConfig {
		return DefaultCircuitBreakerConfig(name)
	})

	cb1 := group.Get("service1")
	cb2 := group.Get("service2")
	cb1Again := group.Get("service1")

	if cb1 != cb1Again {
		t.Error("expected same circuit breaker for same name")
	}
	if cb1 == cb2 {
		t.Error("expected different circuit breakers for different names")
	}

	states := group.States()
	if len(states) != 2 {
		t.Errorf("expected 2 states, got %d", len(states))
	}
}

func TestExecuteTyped(t *testing.T) {
	config := DefaultCircuitBreakerConfig("test")
	cb := NewCircuitBreaker(config)

	type User struct {
		Name string
	}

	user, err := ExecuteTyped(cb, func() (*User, error) {
		return &User{Name: "test"}, nil
	})

	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	if user == nil || user.Name != "test" {
		t.Error("expected user with name 'test'")
	}
}

// =============================================================================
// Timeout Tests
// =============================================================================

func TestWithTimeout_Success(t *testing.T) {
	ctx := context.Background()

	result, err := WithTimeout(ctx, time.Second, func(ctx context.Context) (string, error) {
		return "success", nil
	})

	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	if result != "success" {
		t.Errorf("expected 'success', got %s", result)
	}
}

func TestWithTimeout_Timeout(t *testing.T) {
	ctx := context.Background()

	_, err := WithTimeout(ctx, 50*time.Millisecond, func(ctx context.Context) (string, error) {
		time.Sleep(200 * time.Millisecond)
		return "success", nil
	})

	if err == nil {
		t.Error("expected error, got nil")
	}
	if !IsTimeoutError(err) {
		t.Errorf("expected TimeoutError, got %T", err)
	}
}

func TestWithTimeout_ContextCancellation(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	cancel() // Cancel immediately

	_, err := WithTimeout(ctx, time.Second, func(ctx context.Context) (string, error) {
		return "success", nil
	})

	if err == nil {
		t.Error("expected error, got nil")
	}
}

func TestWithTimeoutFunc(t *testing.T) {
	ctx := context.Background()
	executed := false

	err := WithTimeoutFunc(ctx, time.Second, func(ctx context.Context) error {
		executed = true
		return nil
	})

	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	if !executed {
		t.Error("expected function to be executed")
	}
}

func TestTimeoutRunner(t *testing.T) {
	runner := NewTimeoutRunner(time.Second)
	ctx := context.Background()

	result, err := runner.Run(ctx, func(ctx context.Context) (interface{}, error) {
		return "success", nil
	})

	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	if result != "success" {
		t.Errorf("expected 'success', got %v", result)
	}
}

// =============================================================================
// Bulkhead Tests
// =============================================================================

func TestBulkhead_WithinLimit(t *testing.T) {
	bh := NewBulkheadWithLimit(2)
	ctx := context.Background()

	result, err := bh.Execute(ctx, func() (interface{}, error) {
		return "success", nil
	})

	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	if result != "success" {
		t.Errorf("expected 'success', got %v", result)
	}
}

func TestBulkhead_AtCapacity(t *testing.T) {
	bh := NewBulkheadWithLimit(1)
	ctx := context.Background()

	// Start a long-running operation
	started := make(chan struct{})
	done := make(chan struct{})
	go func() {
		_, _ = bh.Execute(ctx, func() (interface{}, error) {
			close(started)
			<-done
			return "first", nil
		})
	}()

	// Wait for the first operation to start
	<-started

	// Try another operation
	_, err := bh.Execute(ctx, func() (interface{}, error) {
		return "second", nil
	})

	// Clean up
	close(done)

	if err == nil {
		t.Error("expected error, got nil")
	}
	if !IsBulkheadError(err) {
		t.Errorf("expected BulkheadError, got %T", err)
	}
}

func TestBulkhead_WithWaitTime(t *testing.T) {
	config := &BulkheadConfig{
		MaxConcurrent: 1,
		MaxWaitTime:   200 * time.Millisecond,
	}
	bh := NewBulkhead(config)
	ctx := context.Background()

	// Start a short operation
	done := make(chan struct{})
	go func() {
		_, _ = bh.Execute(ctx, func() (interface{}, error) {
			time.Sleep(50 * time.Millisecond)
			return "first", nil
		})
		close(done)
	}()

	// Wait a bit for the first operation to start
	time.Sleep(10 * time.Millisecond)

	// This should wait and succeed
	result, err := bh.Execute(ctx, func() (interface{}, error) {
		return "second", nil
	})

	<-done

	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	if result != "second" {
		t.Errorf("expected 'second', got %v", result)
	}
}

func TestBulkhead_Stats(t *testing.T) {
	bh := NewBulkheadWithLimit(2)
	ctx := context.Background()

	// Execute some operations
	for i := 0; i < 5; i++ {
		_, _ = bh.Execute(ctx, func() (interface{}, error) {
			return "success", nil
		})
	}

	stats := bh.Stats()
	if stats.MaxConcurrent != 2 {
		t.Errorf("expected max concurrent 2, got %d", stats.MaxConcurrent)
	}
	if stats.Total != 5 {
		t.Errorf("expected total 5, got %d", stats.Total)
	}
	if stats.Rejected != 0 {
		t.Errorf("expected no rejections, got %d", stats.Rejected)
	}
}

func TestBulkhead_ConcurrentExecution(t *testing.T) {
	bh := NewBulkheadWithLimit(10)
	ctx := context.Background()

	var wg sync.WaitGroup
	var successCount int64

	for i := 0; i < 20; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			_, err := bh.Execute(ctx, func() (interface{}, error) {
				time.Sleep(10 * time.Millisecond)
				return "success", nil
			})
			if err == nil {
				atomic.AddInt64(&successCount, 1)
			}
		}()
	}

	wg.Wait()

	// Some should succeed, some may be rejected
	if successCount < 10 {
		t.Errorf("expected at least 10 successes, got %d", successCount)
	}
}

func TestBulkheadGroup(t *testing.T) {
	group := NewBulkheadGroup(func(name string) *BulkheadConfig {
		return &BulkheadConfig{MaxConcurrent: 5}
	})
	ctx := context.Background()

	bh1 := group.Get("service1")
	bh2 := group.Get("service2")
	bh1Again := group.Get("service1")

	if bh1 != bh1Again {
		t.Error("expected same bulkhead for same name")
	}
	if bh1 == bh2 {
		t.Error("expected different bulkheads for different names")
	}

	// Execute through the group
	_, err := group.Execute(ctx, "service1", func() (interface{}, error) {
		return "success", nil
	})
	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}

	stats := group.Stats()
	if len(stats) != 2 {
		t.Errorf("expected 2 stats entries, got %d", len(stats))
	}
}

func TestExecuteBulkheadTyped(t *testing.T) {
	bh := NewBulkheadWithLimit(10)
	ctx := context.Background()

	type User struct {
		Name string
	}

	user, err := ExecuteBulkheadTyped(bh, ctx, func() (*User, error) {
		return &User{Name: "test"}, nil
	})

	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	if user == nil || user.Name != "test" {
		t.Error("expected user with name 'test'")
	}
}

// =============================================================================
// Config Validation Tests
// =============================================================================

func TestRetryConfig_Validate(t *testing.T) {
	config := &RetryConfig{}
	validated := config.Validate()

	if validated.MaxAttempts != 3 {
		t.Errorf("expected MaxAttempts 3, got %d", validated.MaxAttempts)
	}
	if validated.InitialInterval != 100*time.Millisecond {
		t.Errorf("expected InitialInterval 100ms, got %v", validated.InitialInterval)
	}
}

func TestCircuitBreakerConfig_Validate(t *testing.T) {
	config := &CircuitBreakerConfig{}
	validated := config.Validate()

	if validated.Name != "default" {
		t.Errorf("expected Name 'default', got %s", validated.Name)
	}
	if validated.FailureThreshold != 5 {
		t.Errorf("expected FailureThreshold 5, got %d", validated.FailureThreshold)
	}
}

func TestBulkheadConfig_Validate(t *testing.T) {
	config := &BulkheadConfig{MaxConcurrent: -1}
	validated := config.Validate()

	if validated.MaxConcurrent != 10 {
		t.Errorf("expected MaxConcurrent 10, got %d", validated.MaxConcurrent)
	}
}
