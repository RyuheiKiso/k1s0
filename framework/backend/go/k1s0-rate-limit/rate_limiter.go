// Package k1s0ratelimit provides rate limiting patterns for k1s0 microservices.
//
// Supported algorithms:
//   - Token bucket: fixed capacity with constant refill rate
//   - Sliding window: request count within a time window
//
// # Usage
//
//	// Token bucket rate limiter
//	config := &k1s0ratelimit.TokenBucketConfig{
//	    Capacity:   100,
//	    RefillRate: 10.0,
//	}
//	limiter := k1s0ratelimit.NewTokenBucket(config)
//
//	if err := limiter.TryAcquire(ctx); err != nil {
//	    // Rate limit exceeded
//	}
//
//	// Sliding window rate limiter
//	swConfig := &k1s0ratelimit.SlidingWindowConfig{
//	    WindowSize:  time.Minute,
//	    MaxRequests: 100,
//	}
//	limiter := k1s0ratelimit.NewSlidingWindow(swConfig)
package k1s0ratelimit

import (
	"context"
	"time"
)

// RateLimiter is the common interface for rate limiting algorithms.
type RateLimiter interface {
	// TryAcquire attempts to acquire permission. Returns nil if allowed, error if rejected.
	TryAcquire(ctx context.Context) error

	// TimeUntilAvailable returns duration until next token is available.
	TimeUntilAvailable() time.Duration

	// AvailableTokens returns current available tokens.
	AvailableTokens() int64

	// Stats returns current rate limiter statistics.
	Stats() Stats
}

// Stats holds rate limiter statistics.
type Stats struct {
	// Allowed is the total number of allowed requests.
	Allowed int64

	// Rejected is the total number of rejected requests.
	Rejected int64

	// Total is the total number of requests.
	Total int64

	// Available is the current number of available tokens.
	Available int64
}

// RejectionRate returns the rejection rate (0.0 to 1.0).
func (s Stats) RejectionRate() float64 {
	if s.Total == 0 {
		return 0
	}
	return float64(s.Rejected) / float64(s.Total)
}
