package k1s0ratelimit

import (
	"context"
	"time"
)

// RateLimitInterceptor provides gRPC/HTTP middleware integration.
type RateLimitInterceptor struct {
	limiter RateLimiter
}

// NewRateLimitInterceptor creates a new RateLimitInterceptor.
//
// Example:
//
//	limiter := k1s0ratelimit.NewTokenBucket(config)
//	interceptor := k1s0ratelimit.NewRateLimitInterceptor(limiter)
//
//	if err := interceptor.Check(ctx); err != nil {
//	    // Return gRPC ResourceExhausted or HTTP 429
//	}
func NewRateLimitInterceptor(limiter RateLimiter) *RateLimitInterceptor {
	return &RateLimitInterceptor{
		limiter: limiter,
	}
}

// Check performs a rate limit check. Returns nil if allowed, error if rejected.
func (i *RateLimitInterceptor) Check(ctx context.Context) error {
	return i.limiter.TryAcquire(ctx)
}

// RetryAfter returns the recommended retry-after duration.
func (i *RateLimitInterceptor) RetryAfter() time.Duration {
	return i.limiter.TimeUntilAvailable()
}

// Stats returns the current rate limiter statistics.
func (i *RateLimitInterceptor) Stats() Stats {
	return i.limiter.Stats()
}
