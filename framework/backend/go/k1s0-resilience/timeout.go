package k1s0resilience

import (
	"context"
	"time"
)

// TimeoutError represents an error when an operation times out.
type TimeoutError struct {
	// Duration is the timeout duration that was exceeded.
	Duration time.Duration

	// OriginalError is the original context error (context.DeadlineExceeded).
	OriginalError error
}

// Error implements the error interface.
func (e *TimeoutError) Error() string {
	return "operation timed out after " + e.Duration.String()
}

// Unwrap returns the original error for errors.Is/As support.
func (e *TimeoutError) Unwrap() error {
	return e.OriginalError
}

// WithTimeout executes the given function with a timeout.
// If the function doesn't complete within the timeout, it returns a TimeoutError.
//
// Note: The function should respect the context cancellation by checking ctx.Done().
// If the function doesn't check the context, it will continue running in the background
// even after the timeout, but the caller will receive a timeout error.
//
// Example:
//
//	result, err := k1s0resilience.WithTimeout(ctx, 5*time.Second, func(ctx context.Context) (string, error) {
//	    return httpClient.DoWithContext(ctx, req)
//	})
func WithTimeout[T any](ctx context.Context, timeout time.Duration, fn func(ctx context.Context) (T, error)) (T, error) {
	var zero T

	// Create a context with timeout
	timeoutCtx, cancel := context.WithTimeout(ctx, timeout)
	defer cancel()

	// Channel to receive the result
	type result struct {
		value T
		err   error
	}
	resultCh := make(chan result, 1)

	// Execute the function in a goroutine
	go func() {
		value, err := fn(timeoutCtx)
		resultCh <- result{value: value, err: err}
	}()

	// Wait for result or timeout
	select {
	case r := <-resultCh:
		return r.value, r.err
	case <-timeoutCtx.Done():
		err := timeoutCtx.Err()
		if err == context.DeadlineExceeded {
			return zero, &TimeoutError{Duration: timeout, OriginalError: err}
		}
		return zero, err
	}
}

// WithTimeoutFunc executes the given function with a timeout (no return value version).
// This is a convenience wrapper for functions that don't return a value.
//
// Example:
//
//	err := k1s0resilience.WithTimeoutFunc(ctx, 5*time.Second, func(ctx context.Context) error {
//	    return client.Ping(ctx)
//	})
func WithTimeoutFunc(ctx context.Context, timeout time.Duration, fn func(ctx context.Context) error) error {
	_, err := WithTimeout(ctx, timeout, func(ctx context.Context) (struct{}, error) {
		return struct{}{}, fn(ctx)
	})
	return err
}

// IsTimeoutError checks if the error is a TimeoutError.
func IsTimeoutError(err error) bool {
	_, ok := err.(*TimeoutError)
	return ok
}

// TimeoutRunner provides a reusable timeout wrapper.
type TimeoutRunner struct {
	timeout time.Duration
}

// NewTimeoutRunner creates a new TimeoutRunner with the given timeout.
func NewTimeoutRunner(timeout time.Duration) *TimeoutRunner {
	return &TimeoutRunner{timeout: timeout}
}

// Run executes the given function with the configured timeout.
func (r *TimeoutRunner) Run(ctx context.Context, fn func(ctx context.Context) (interface{}, error)) (interface{}, error) {
	return WithTimeout(ctx, r.timeout, fn)
}

// RunTyped executes the given function with the configured timeout and type safety.
func RunTyped[T any](r *TimeoutRunner, ctx context.Context, fn func(ctx context.Context) (T, error)) (T, error) {
	return WithTimeout(ctx, r.timeout, fn)
}
