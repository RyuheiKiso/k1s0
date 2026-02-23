package resiliency

import (
	"context"
	"errors"
	"fmt"
	"math"
	"sync"
	"sync/atomic"
	"time"
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

type circuitState int

const (
	circuitClosed circuitState = iota
	circuitOpen
	circuitHalfOpen
)

// ResiliencyDecorator applies a ResiliencyPolicy to function executions.
type ResiliencyDecorator struct {
	policy          ResiliencyPolicy
	bulkheadCh      chan struct{}
	cbState         circuitState
	cbMu            sync.Mutex
	cbFailureCount  atomic.Int32
	cbSuccessCount  atomic.Int32
	cbLastFailureAt time.Time
}

// NewResiliencyDecorator creates a new decorator from the given policy.
func NewResiliencyDecorator(policy ResiliencyPolicy) *ResiliencyDecorator {
	d := &ResiliencyDecorator{
		policy: policy,
	}
	if policy.Bulkhead != nil {
		d.bulkheadCh = make(chan struct{}, policy.Bulkhead.MaxConcurrentCalls)
		for i := 0; i < policy.Bulkhead.MaxConcurrentCalls; i++ {
			d.bulkheadCh <- struct{}{}
		}
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
	if d.bulkheadCh != nil {
		cfg := d.policy.Bulkhead
		select {
		case <-d.bulkheadCh:
			defer func() { d.bulkheadCh <- struct{}{} }()
		case <-time.After(cfg.MaxWaitDuration):
			return zero, &ResiliencyError{
				Kind:    "bulkhead_full",
				Message: fmt.Sprintf("max concurrent calls: %d", cfg.MaxConcurrentCalls),
				Cause:   ErrBulkheadFull,
			}
		case <-ctx.Done():
			return zero, ctx.Err()
		}
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
	if d.policy.CircuitBreaker == nil {
		return nil
	}

	d.cbMu.Lock()
	defer d.cbMu.Unlock()

	switch d.cbState {
	case circuitClosed:
		return nil
	case circuitOpen:
		if time.Since(d.cbLastFailureAt) >= d.policy.CircuitBreaker.RecoveryTimeout {
			d.cbState = circuitHalfOpen
			d.cbSuccessCount.Store(0)
			return nil
		}
		remaining := d.policy.CircuitBreaker.RecoveryTimeout - time.Since(d.cbLastFailureAt)
		return &ResiliencyError{
			Kind:    "circuit_open",
			Message: fmt.Sprintf("circuit breaker open, remaining: %v", remaining),
			Cause:   ErrCircuitBreakerOpen,
		}
	case circuitHalfOpen:
		return nil
	}
	return nil
}

func (d *ResiliencyDecorator) recordSuccess() {
	if d.policy.CircuitBreaker == nil {
		return
	}

	d.cbMu.Lock()
	defer d.cbMu.Unlock()

	switch d.cbState {
	case circuitHalfOpen:
		count := d.cbSuccessCount.Add(1)
		if int(count) >= d.policy.CircuitBreaker.HalfOpenMaxCalls {
			d.cbState = circuitClosed
			d.cbFailureCount.Store(0)
		}
	case circuitClosed:
		d.cbFailureCount.Store(0)
	}
}

func (d *ResiliencyDecorator) recordFailure() {
	if d.policy.CircuitBreaker == nil {
		return
	}

	count := d.cbFailureCount.Add(1)
	if int(count) >= d.policy.CircuitBreaker.FailureThreshold {
		d.cbMu.Lock()
		d.cbState = circuitOpen
		d.cbLastFailureAt = time.Now()
		d.cbMu.Unlock()
	}
}

func calculateBackoff(attempt int, baseDelay, maxDelay time.Duration) time.Duration {
	delay := baseDelay * time.Duration(math.Pow(2, float64(attempt)))
	if delay > maxDelay {
		delay = maxDelay
	}
	return delay
}
