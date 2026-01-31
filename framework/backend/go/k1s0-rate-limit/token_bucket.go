package k1s0ratelimit

import (
	"context"
	"sync"
	"sync/atomic"
	"time"
)

// TokenBucket implements the token bucket rate limiting algorithm.
//
// Tokens are added at a constant refill rate up to a maximum capacity.
// Each request consumes one token. If no tokens are available, the request is rejected.
type TokenBucket struct {
	config     *TokenBucketConfig
	tokens     int64
	lastRefill int64 // unix nanoseconds
	allowed    int64
	rejected   int64
	total      int64
	mu         sync.Mutex
}

// NewTokenBucket creates a new TokenBucket with the given configuration.
//
// Example:
//
//	config := &k1s0ratelimit.TokenBucketConfig{
//	    Capacity:   100,
//	    RefillRate: 10.0,
//	}
//	limiter := k1s0ratelimit.NewTokenBucket(config)
func NewTokenBucket(config *TokenBucketConfig) *TokenBucket {
	config = config.Validate()
	now := time.Now().UnixNano()
	return &TokenBucket{
		config:     config,
		tokens:     config.Capacity,
		lastRefill: now,
	}
}

// refill adds tokens based on elapsed time since last refill.
// Must be called with mu held.
func (tb *TokenBucket) refill() {
	now := time.Now().UnixNano()
	last := tb.lastRefill
	elapsed := time.Duration(now - last)

	tokensToAdd := int64(elapsed.Seconds() * tb.config.RefillRate)
	if tokensToAdd <= 0 {
		return
	}

	tb.lastRefill = now
	newTokens := tb.tokens + tokensToAdd
	if newTokens > tb.config.Capacity {
		newTokens = tb.config.Capacity
	}
	tb.tokens = newTokens
}

// TryAcquire attempts to acquire a token. Returns nil if allowed, error if rejected.
func (tb *TokenBucket) TryAcquire(ctx context.Context) error {
	if ctx.Err() != nil {
		return ctx.Err()
	}

	atomic.AddInt64(&tb.total, 1)

	tb.mu.Lock()
	tb.refill()

	if tb.tokens > 0 {
		tb.tokens--
		tb.mu.Unlock()
		atomic.AddInt64(&tb.allowed, 1)
		return nil
	}

	tb.mu.Unlock()
	atomic.AddInt64(&tb.rejected, 1)
	return &ErrRateLimitExceeded{
		RetryAfter: tb.TimeUntilAvailable(),
	}
}

// TimeUntilAvailable returns the duration until the next token is available.
func (tb *TokenBucket) TimeUntilAvailable() time.Duration {
	tb.mu.Lock()
	tokens := tb.tokens
	tb.mu.Unlock()

	if tokens > 0 {
		return 0
	}

	// Time for one token to be refilled
	return time.Duration(float64(time.Second) / tb.config.RefillRate)
}

// AvailableTokens returns the current number of available tokens.
func (tb *TokenBucket) AvailableTokens() int64 {
	tb.mu.Lock()
	tb.refill()
	tokens := tb.tokens
	tb.mu.Unlock()
	return tokens
}

// Stats returns the current rate limiter statistics.
func (tb *TokenBucket) Stats() Stats {
	return Stats{
		Allowed:   atomic.LoadInt64(&tb.allowed),
		Rejected:  atomic.LoadInt64(&tb.rejected),
		Total:     atomic.LoadInt64(&tb.total),
		Available: tb.AvailableTokens(),
	}
}
