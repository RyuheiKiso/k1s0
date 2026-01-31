package k1s0ratelimit

import (
	"context"
	"sync"
	"sync/atomic"
	"time"
)

// SlidingWindow implements the sliding window rate limiting algorithm.
//
// Requests are tracked with timestamps. When a new request arrives, timestamps
// outside the window are pruned, and the request is allowed only if the count
// is below the maximum.
type SlidingWindow struct {
	config     *SlidingWindowConfig
	timestamps []time.Time
	allowed    int64
	rejected   int64
	total      int64
	mu         sync.Mutex
}

// NewSlidingWindow creates a new SlidingWindow with the given configuration.
//
// Example:
//
//	config := &k1s0ratelimit.SlidingWindowConfig{
//	    WindowSize:  time.Minute,
//	    MaxRequests: 100,
//	}
//	limiter := k1s0ratelimit.NewSlidingWindow(config)
func NewSlidingWindow(config *SlidingWindowConfig) *SlidingWindow {
	config = config.Validate()
	return &SlidingWindow{
		config:     config,
		timestamps: make([]time.Time, 0, config.MaxRequests),
	}
}

// prune removes timestamps outside the current window.
// Must be called with mu held.
func (sw *SlidingWindow) prune(now time.Time) {
	cutoff := now.Add(-sw.config.WindowSize)
	i := 0
	for i < len(sw.timestamps) && sw.timestamps[i].Before(cutoff) {
		i++
	}
	if i > 0 {
		sw.timestamps = sw.timestamps[i:]
	}
}

// TryAcquire attempts to acquire permission. Returns nil if allowed, error if rejected.
func (sw *SlidingWindow) TryAcquire(ctx context.Context) error {
	if ctx.Err() != nil {
		return ctx.Err()
	}

	atomic.AddInt64(&sw.total, 1)
	now := time.Now()

	sw.mu.Lock()
	sw.prune(now)

	if int64(len(sw.timestamps)) < sw.config.MaxRequests {
		sw.timestamps = append(sw.timestamps, now)
		sw.mu.Unlock()
		atomic.AddInt64(&sw.allowed, 1)
		return nil
	}

	// Calculate retry-after: time until the oldest entry expires
	var retryAfter time.Duration
	if len(sw.timestamps) > 0 {
		oldest := sw.timestamps[0]
		retryAfter = sw.config.WindowSize - now.Sub(oldest)
		if retryAfter < 0 {
			retryAfter = 0
		}
	}
	sw.mu.Unlock()

	atomic.AddInt64(&sw.rejected, 1)
	return &ErrRateLimitExceeded{
		RetryAfter: retryAfter,
	}
}

// TimeUntilAvailable returns the duration until the next request can be made.
func (sw *SlidingWindow) TimeUntilAvailable() time.Duration {
	now := time.Now()
	sw.mu.Lock()
	sw.prune(now)

	if int64(len(sw.timestamps)) < sw.config.MaxRequests {
		sw.mu.Unlock()
		return 0
	}

	var retryAfter time.Duration
	if len(sw.timestamps) > 0 {
		oldest := sw.timestamps[0]
		retryAfter = sw.config.WindowSize - now.Sub(oldest)
		if retryAfter < 0 {
			retryAfter = 0
		}
	}
	sw.mu.Unlock()
	return retryAfter
}

// AvailableTokens returns the number of requests available in the current window.
func (sw *SlidingWindow) AvailableTokens() int64 {
	now := time.Now()
	sw.mu.Lock()
	sw.prune(now)
	available := sw.config.MaxRequests - int64(len(sw.timestamps))
	sw.mu.Unlock()
	if available < 0 {
		return 0
	}
	return available
}

// Stats returns the current rate limiter statistics.
func (sw *SlidingWindow) Stats() Stats {
	return Stats{
		Allowed:   atomic.LoadInt64(&sw.allowed),
		Rejected:  atomic.LoadInt64(&sw.rejected),
		Total:     atomic.LoadInt64(&sw.total),
		Available: sw.AvailableTokens(),
	}
}
