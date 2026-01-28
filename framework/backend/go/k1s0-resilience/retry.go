package k1s0resilience

import (
	"context"
	"fmt"
	"math"
	"math/rand"
	"time"
)

// RetryError represents an error that occurred during retry operations.
type RetryError struct {
	// Attempts is the number of attempts made.
	Attempts int

	// LastError is the error from the last attempt.
	LastError error
}

// Error implements the error interface.
func (e *RetryError) Error() string {
	return fmt.Sprintf("retry failed after %d attempts: %v", e.Attempts, e.LastError)
}

// Unwrap returns the underlying error for errors.Is/As support.
func (e *RetryError) Unwrap() error {
	return e.LastError
}

// Retry executes the given function with exponential backoff retry logic.
// It returns the result of the function or a RetryError if all attempts fail.
//
// The function will be retried according to the configuration:
//   - MaxAttempts: maximum number of attempts (including the initial call)
//   - InitialInterval: initial backoff interval
//   - MaxInterval: maximum backoff interval
//   - Multiplier: exponential backoff multiplier
//   - JitterFactor: random jitter factor (0.0 to 1.0)
//
// Example:
//
//	config := k1s0resilience.DefaultRetryConfig()
//	config.MaxAttempts = 5
//
//	result, err := k1s0resilience.Retry(ctx, config, func() (string, error) {
//	    return apiClient.Call()
//	})
func Retry[T any](ctx context.Context, config *RetryConfig, fn func() (T, error)) (T, error) {
	config = config.Validate()

	var zero T
	var lastErr error

	for attempt := 1; attempt <= config.MaxAttempts; attempt++ {
		// Check context before attempting
		select {
		case <-ctx.Done():
			if lastErr != nil {
				return zero, &RetryError{Attempts: attempt - 1, LastError: lastErr}
			}
			return zero, ctx.Err()
		default:
		}

		// Execute the function
		result, err := fn()
		if err == nil {
			return result, nil
		}

		lastErr = err

		// Check if error is retryable
		if config.RetryableChecker != nil && !config.RetryableChecker(err) {
			return zero, &RetryError{Attempts: attempt, LastError: err}
		}

		// If this was the last attempt, don't sleep
		if attempt == config.MaxAttempts {
			break
		}

		// Calculate backoff with jitter
		backoff := calculateBackoff(attempt, config)

		// Wait for backoff or context cancellation
		select {
		case <-ctx.Done():
			return zero, &RetryError{Attempts: attempt, LastError: lastErr}
		case <-time.After(backoff):
		}
	}

	return zero, &RetryError{Attempts: config.MaxAttempts, LastError: lastErr}
}

// RetryWithResult is an alias for Retry for backward compatibility.
var RetryWithResult = Retry[any]

// calculateBackoff calculates the backoff duration with exponential growth and jitter.
func calculateBackoff(attempt int, config *RetryConfig) time.Duration {
	// Calculate exponential backoff
	backoff := float64(config.InitialInterval) * math.Pow(config.Multiplier, float64(attempt-1))

	// Cap at max interval
	if backoff > float64(config.MaxInterval) {
		backoff = float64(config.MaxInterval)
	}

	// Apply jitter
	if config.JitterFactor > 0 {
		jitter := backoff * config.JitterFactor * (rand.Float64()*2 - 1)
		backoff += jitter
	}

	// Ensure non-negative
	if backoff < 0 {
		backoff = 0
	}

	return time.Duration(backoff)
}

// RetryFunc executes the given function with retry logic (no return value version).
// This is a convenience wrapper for functions that don't return a value.
//
// Example:
//
//	err := k1s0resilience.RetryFunc(ctx, config, func() error {
//	    return apiClient.Ping()
//	})
func RetryFunc(ctx context.Context, config *RetryConfig, fn func() error) error {
	_, err := Retry(ctx, config, func() (struct{}, error) {
		return struct{}{}, fn()
	})
	return err
}

// IsRetryError checks if the error is a RetryError.
func IsRetryError(err error) bool {
	_, ok := err.(*RetryError)
	return ok
}

// GetRetryAttempts returns the number of retry attempts from a RetryError.
// Returns 0 if the error is not a RetryError.
func GetRetryAttempts(err error) int {
	if retryErr, ok := err.(*RetryError); ok {
		return retryErr.Attempts
	}
	return 0
}
