package k1s0ratelimit

import (
	"errors"
	"fmt"
	"time"
)

// ErrRateLimitExceeded is returned when the rate limit is exceeded.
type ErrRateLimitExceeded struct {
	// RetryAfter is the recommended duration to wait before retrying.
	RetryAfter time.Duration
}

// Error implements the error interface.
func (e *ErrRateLimitExceeded) Error() string {
	return fmt.Sprintf("rate limit exceeded, retry after %v", e.RetryAfter)
}

// IsRateLimitExceeded checks if the error is an ErrRateLimitExceeded.
func IsRateLimitExceeded(err error) bool {
	var rlErr *ErrRateLimitExceeded
	return errors.As(err, &rlErr)
}

// GetRetryAfter extracts the retry-after duration from a rate limit error.
// Returns zero duration if the error is not a rate limit error.
func GetRetryAfter(err error) time.Duration {
	var rlErr *ErrRateLimitExceeded
	if errors.As(err, &rlErr) {
		return rlErr.RetryAfter
	}
	return 0
}
