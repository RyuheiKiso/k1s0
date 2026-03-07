package resiliency

import (
	"context"
	"errors"
	"fmt"
	"math"
	"time"

	"github.com/k1s0-platform/system-library-go-bulkhead"
	"github.com/k1s0-platform/system-library-go-circuit-breaker"
)

// Error types
var (
	ErrMaxRetriesExceeded = errors.New("max retries exceeded")
	ErrCircuitBreakerOpen = errors.New("circuit breaker is open")
	ErrBulkheadFull       = errors.New("bulkhead full")
	ErrTimeout            = errors.New("operation timed out")
)

// ResiliencyError wraps an error with resiliency context.
type ResiliencyError struct {
	Kind    string
	Message string
	Cause   error
}

func (e *ResiliencyError) Error() string {
	if e.Cause != nil {
		return fmt.Sprintf("%s: %s: %v", e.Kind, e.Message, e.Cause)
	}
	return fmt.Sprintf("%s: %s", e.Kind, e.Message)
}

func (e *ResiliencyError) Unwrap() error {
	return e.Cause
}

// RetryConfig configures retry behavior.
type RetryConfig struct {
	MaxAttempts int
	BaseDelay   time.Duration
	MaxDelay    time.Duration
	Jitter      bool
}

// CircuitBreakerConfig configures circuit breaker behavior.
type CircuitBreakerConfig struct {
	FailureThreshold int
	RecoveryTimeout  time.Duration
	HalfOpenMaxCalls int
}

// BulkheadConfig configures bulkhead (concurrency limiter) behavior.
type BulkheadConfig struct {
	MaxConcurrentCalls int
	MaxWaitDuration    time.Duration
}

// ResiliencyPolicy combines retry, circuit breaker, bulkhead, and timeout.
type ResiliencyPolicy struct {
	Retry          *RetryConfig
	CircuitBreaker *CircuitBreakerConfig
	Bulkhead       *BulkheadConfig
	Timeout        time.Duration
}

// ResiliencyDecorator applies a ResiliencyPolicy to function executions.
type ResiliencyDecorator struct {
	policy ResiliencyPolicy
	bh     *bulkhead.Bulkhead
	cb     *circuitbreaker.CircuitBreaker
}

// NewResiliencyDecorator creates a new decorator from the given policy.
func NewResiliencyDecorator(policy ResiliencyPolicy) *ResiliencyDecorator {
	d := &ResiliencyDecorator{
		policy: policy,
	}
	if policy.Bulkhead != nil {
		d.bh = bulkhead.New(bulkhead.Config{
			MaxConcurrentCalls: policy.Bulkhead.MaxConcurrentCalls,
			MaxWaitDuration:    policy.Bulkhead.MaxWaitDuration,
		})
	}
	if policy.CircuitBreaker != nil {
		d.cb = circuitbreaker.New(circuitbreaker.Config{
			FailureThreshold: uint32(policy.CircuitBreaker.FailureThreshold),
			SuccessThreshold: uint32(policy.CircuitBreaker.HalfOpenMaxCalls),
			Timeout:          policy.CircuitBreaker.RecoveryTimeout,
		})
	}
	return d
}

// Execute runs the provided function with the configured resiliency policy.
func Execute[T any](ctx context.Context, d *ResiliencyDecorator, fn func() (T, error)) (T, error) {
	var zero T

	// Check circuit breaker
	if err := d.checkCircuitBreaker(); err != nil {
		return zero, err
	}

	// Acquire bulkhead permit
	if d.bh != nil {
		err := d.bh.Acquire(ctx)
		if err != nil {
			if errors.Is(err, bulkhead.ErrFull) {
				return zero, &ResiliencyError{
					Kind:    "bulkhead_full",
					Message: fmt.Sprintf("max concurrent calls: %d", d.policy.Bulkhead.MaxConcurrentCalls),
					Cause:   ErrBulkheadFull,
				}
			}
			return zero, err
		}
		defer d.bh.Release()
	}

	maxAttempts := 1
	if d.policy.Retry != nil {
		maxAttempts = d.policy.Retry.MaxAttempts
	}

	var lastErr error
	for attempt := 0; attempt < maxAttempts; attempt++ {
		result, err := executeWithTimeout(ctx, d, fn)
		if err == nil {
			d.recordSuccess()
			return result, nil
		}

		// If the error is a resiliency error (timeout, circuit open, bulkhead full),
		// return it directly without wrapping in max_retries_exceeded
		var rErr *ResiliencyError
		if errors.As(err, &rErr) {
			return zero, err
		}

		d.recordFailure()
		lastErr = err

		if cbErr := d.checkCircuitBreaker(); cbErr != nil {
			return zero, cbErr
		}

		if attempt+1 < maxAttempts && d.policy.Retry != nil {
			delay := calculateBackoff(attempt, d.policy.Retry.BaseDelay, d.policy.Retry.MaxDelay)
			select {
			case <-time.After(delay):
			case <-ctx.Done():
				return zero, ctx.Err()
			}
		}
	}

	return zero, &ResiliencyError{
		Kind:    "max_retries_exceeded",
		Message: fmt.Sprintf("failed after %d attempts", maxAttempts),
		Cause:   lastErr,
	}
}

func executeWithTimeout[T any](ctx context.Context, d *ResiliencyDecorator, fn func() (T, error)) (T, error) {
	var zero T
	if d.policy.Timeout <= 0 {
		return fn()
	}

	ctx, cancel := context.WithTimeout(ctx, d.policy.Timeout)
	defer cancel()

	type result struct {
		val T
		err error
	}

	ch := make(chan result, 1)
	go func() {
		v, e := fn()
		ch <- result{v, e}
	}()

	select {
	case r := <-ch:
		return r.val, r.err
	case <-ctx.Done():
		return zero, &ResiliencyError{
			Kind:    "timeout",
			Message: fmt.Sprintf("timed out after %v", d.policy.Timeout),
			Cause:   ErrTimeout,
		}
	}
}

func (d *ResiliencyDecorator) checkCircuitBreaker() error {
	if d.cb == nil {
		return nil
	}

	state := d.cb.State()
	if state == circuitbreaker.StateOpen {
		return &ResiliencyError{
			Kind:    "circuit_open",
			Message: "circuit breaker open",
			Cause:   ErrCircuitBreakerOpen,
		}
	}
	return nil
}

func (d *ResiliencyDecorator) recordSuccess() {
	if d.cb != nil {
		d.cb.RecordSuccess()
	}
}

func (d *ResiliencyDecorator) recordFailure() {
	if d.cb != nil {
		d.cb.RecordFailure()
	}
}

func calculateBackoff(attempt int, baseDelay, maxDelay time.Duration) time.Duration {
	delay := baseDelay * time.Duration(math.Pow(2, float64(attempt)))
	if delay > maxDelay {
		delay = maxDelay
	}
	return delay
}
