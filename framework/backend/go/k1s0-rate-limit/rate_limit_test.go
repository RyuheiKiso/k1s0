package k1s0ratelimit

import (
	"context"
	"sync"
	"sync/atomic"
	"testing"
	"time"
)

// =============================================================================
// Token Bucket Tests
// =============================================================================

func TestTokenBucket_WithinLimit(t *testing.T) {
	config := &TokenBucketConfig{
		Capacity:   10,
		RefillRate: 10.0,
	}
	tb := NewTokenBucket(config)
	ctx := context.Background()

	for i := 0; i < 10; i++ {
		if err := tb.TryAcquire(ctx); err != nil {
			t.Errorf("expected no error on request %d, got %v", i, err)
		}
	}

	stats := tb.Stats()
	if stats.Allowed != 10 {
		t.Errorf("expected 10 allowed, got %d", stats.Allowed)
	}
	if stats.Rejected != 0 {
		t.Errorf("expected 0 rejected, got %d", stats.Rejected)
	}
}

func TestTokenBucket_ExceedsLimit(t *testing.T) {
	config := &TokenBucketConfig{
		Capacity:   5,
		RefillRate: 1.0,
	}
	tb := NewTokenBucket(config)
	ctx := context.Background()

	// Consume all tokens
	for i := 0; i < 5; i++ {
		if err := tb.TryAcquire(ctx); err != nil {
			t.Errorf("expected no error on request %d, got %v", i, err)
		}
	}

	// Next request should fail
	err := tb.TryAcquire(ctx)
	if err == nil {
		t.Error("expected error, got nil")
	}
	if !IsRateLimitExceeded(err) {
		t.Errorf("expected ErrRateLimitExceeded, got %T", err)
	}

	stats := tb.Stats()
	if stats.Allowed != 5 {
		t.Errorf("expected 5 allowed, got %d", stats.Allowed)
	}
	if stats.Rejected != 1 {
		t.Errorf("expected 1 rejected, got %d", stats.Rejected)
	}
	if stats.Total != 6 {
		t.Errorf("expected 6 total, got %d", stats.Total)
	}
}

func TestTokenBucket_Refill(t *testing.T) {
	config := &TokenBucketConfig{
		Capacity:   5,
		RefillRate: 1000.0, // 1000 tokens/sec for fast test
	}
	tb := NewTokenBucket(config)
	ctx := context.Background()

	// Consume all tokens
	for i := 0; i < 5; i++ {
		_ = tb.TryAcquire(ctx)
	}

	// Should be rejected
	if err := tb.TryAcquire(ctx); err == nil {
		t.Error("expected error after consuming all tokens")
	}

	// Wait for refill
	time.Sleep(10 * time.Millisecond)

	// Should be allowed again
	if err := tb.TryAcquire(ctx); err != nil {
		t.Errorf("expected success after refill, got %v", err)
	}
}

func TestTokenBucket_Concurrent(t *testing.T) {
	config := &TokenBucketConfig{
		Capacity:   100,
		RefillRate: 1.0,
	}
	tb := NewTokenBucket(config)
	ctx := context.Background()

	var wg sync.WaitGroup
	var allowed int64
	var rejected int64

	for i := 0; i < 200; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			if err := tb.TryAcquire(ctx); err == nil {
				atomic.AddInt64(&allowed, 1)
			} else {
				atomic.AddInt64(&rejected, 1)
			}
		}()
	}

	wg.Wait()

	if allowed+rejected != 200 {
		t.Errorf("expected 200 total, got %d", allowed+rejected)
	}
	if allowed > 100 {
		t.Errorf("expected at most 100 allowed, got %d", allowed)
	}

	stats := tb.Stats()
	if stats.Total != 200 {
		t.Errorf("expected 200 total in stats, got %d", stats.Total)
	}
}

func TestTokenBucket_ContextCancelled(t *testing.T) {
	config := DefaultTokenBucketConfig()
	tb := NewTokenBucket(config)
	ctx, cancel := context.WithCancel(context.Background())
	cancel()

	err := tb.TryAcquire(ctx)
	if err == nil {
		t.Error("expected error for cancelled context")
	}
}

func TestTokenBucket_TimeUntilAvailable(t *testing.T) {
	config := &TokenBucketConfig{
		Capacity:   1,
		RefillRate: 10.0,
	}
	tb := NewTokenBucket(config)

	// Before consuming: should be zero
	if d := tb.TimeUntilAvailable(); d != 0 {
		t.Errorf("expected 0, got %v", d)
	}

	// Consume the token
	_ = tb.TryAcquire(context.Background())

	// After consuming: should be ~100ms (1/10.0)
	d := tb.TimeUntilAvailable()
	if d < 50*time.Millisecond || d > 150*time.Millisecond {
		t.Errorf("expected ~100ms, got %v", d)
	}
}

func TestTokenBucket_AvailableTokens(t *testing.T) {
	config := &TokenBucketConfig{
		Capacity:   10,
		RefillRate: 1.0,
	}
	tb := NewTokenBucket(config)

	if tokens := tb.AvailableTokens(); tokens != 10 {
		t.Errorf("expected 10, got %d", tokens)
	}

	_ = tb.TryAcquire(context.Background())
	_ = tb.TryAcquire(context.Background())

	if tokens := tb.AvailableTokens(); tokens != 8 {
		t.Errorf("expected 8, got %d", tokens)
	}
}

// =============================================================================
// Sliding Window Tests
// =============================================================================

func TestSlidingWindow_WithinLimit(t *testing.T) {
	config := &SlidingWindowConfig{
		WindowSize:  time.Minute,
		MaxRequests: 10,
	}
	sw := NewSlidingWindow(config)
	ctx := context.Background()

	for i := 0; i < 10; i++ {
		if err := sw.TryAcquire(ctx); err != nil {
			t.Errorf("expected no error on request %d, got %v", i, err)
		}
	}

	stats := sw.Stats()
	if stats.Allowed != 10 {
		t.Errorf("expected 10 allowed, got %d", stats.Allowed)
	}
}

func TestSlidingWindow_ExceedsLimit(t *testing.T) {
	config := &SlidingWindowConfig{
		WindowSize:  time.Minute,
		MaxRequests: 5,
	}
	sw := NewSlidingWindow(config)
	ctx := context.Background()

	for i := 0; i < 5; i++ {
		_ = sw.TryAcquire(ctx)
	}

	err := sw.TryAcquire(ctx)
	if err == nil {
		t.Error("expected error, got nil")
	}
	if !IsRateLimitExceeded(err) {
		t.Errorf("expected ErrRateLimitExceeded, got %T", err)
	}

	stats := sw.Stats()
	if stats.Rejected != 1 {
		t.Errorf("expected 1 rejected, got %d", stats.Rejected)
	}
}

func TestSlidingWindow_WindowExpiry(t *testing.T) {
	config := &SlidingWindowConfig{
		WindowSize:  50 * time.Millisecond,
		MaxRequests: 5,
	}
	sw := NewSlidingWindow(config)
	ctx := context.Background()

	// Fill the window
	for i := 0; i < 5; i++ {
		_ = sw.TryAcquire(ctx)
	}

	// Should be rejected
	if err := sw.TryAcquire(ctx); err == nil {
		t.Error("expected error after filling window")
	}

	// Wait for window to expire
	time.Sleep(60 * time.Millisecond)

	// Should be allowed again
	if err := sw.TryAcquire(ctx); err != nil {
		t.Errorf("expected success after window expiry, got %v", err)
	}
}

func TestSlidingWindow_Concurrent(t *testing.T) {
	config := &SlidingWindowConfig{
		WindowSize:  time.Minute,
		MaxRequests: 50,
	}
	sw := NewSlidingWindow(config)
	ctx := context.Background()

	var wg sync.WaitGroup
	var allowed int64
	var rejected int64

	for i := 0; i < 100; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			if err := sw.TryAcquire(ctx); err == nil {
				atomic.AddInt64(&allowed, 1)
			} else {
				atomic.AddInt64(&rejected, 1)
			}
		}()
	}

	wg.Wait()

	if allowed+rejected != 100 {
		t.Errorf("expected 100 total, got %d", allowed+rejected)
	}
	if allowed > 50 {
		t.Errorf("expected at most 50 allowed, got %d", allowed)
	}

	stats := sw.Stats()
	if stats.Total != 100 {
		t.Errorf("expected 100 total in stats, got %d", stats.Total)
	}
}

func TestSlidingWindow_ContextCancelled(t *testing.T) {
	config := DefaultSlidingWindowConfig()
	sw := NewSlidingWindow(config)
	ctx, cancel := context.WithCancel(context.Background())
	cancel()

	err := sw.TryAcquire(ctx)
	if err == nil {
		t.Error("expected error for cancelled context")
	}
}

func TestSlidingWindow_TimeUntilAvailable(t *testing.T) {
	config := &SlidingWindowConfig{
		WindowSize:  100 * time.Millisecond,
		MaxRequests: 1,
	}
	sw := NewSlidingWindow(config)

	// Before consuming: should be zero
	if d := sw.TimeUntilAvailable(); d != 0 {
		t.Errorf("expected 0, got %v", d)
	}

	_ = sw.TryAcquire(context.Background())

	// After consuming: should be > 0
	d := sw.TimeUntilAvailable()
	if d <= 0 {
		t.Errorf("expected positive duration, got %v", d)
	}
}

func TestSlidingWindow_AvailableTokens(t *testing.T) {
	config := &SlidingWindowConfig{
		WindowSize:  time.Minute,
		MaxRequests: 10,
	}
	sw := NewSlidingWindow(config)

	if tokens := sw.AvailableTokens(); tokens != 10 {
		t.Errorf("expected 10, got %d", tokens)
	}

	_ = sw.TryAcquire(context.Background())
	_ = sw.TryAcquire(context.Background())

	if tokens := sw.AvailableTokens(); tokens != 8 {
		t.Errorf("expected 8, got %d", tokens)
	}
}

// =============================================================================
// Config Validation Tests
// =============================================================================

func TestTokenBucketConfig_Validate(t *testing.T) {
	config := &TokenBucketConfig{Capacity: -1, RefillRate: -1}
	validated := config.Validate()

	if validated.Capacity != 100 {
		t.Errorf("expected Capacity 100, got %d", validated.Capacity)
	}
	if validated.RefillRate != 10.0 {
		t.Errorf("expected RefillRate 10.0, got %f", validated.RefillRate)
	}
}

func TestSlidingWindowConfig_Validate(t *testing.T) {
	config := &SlidingWindowConfig{WindowSize: -1, MaxRequests: -1}
	validated := config.Validate()

	if validated.WindowSize != time.Minute {
		t.Errorf("expected WindowSize 1m, got %v", validated.WindowSize)
	}
	if validated.MaxRequests != 100 {
		t.Errorf("expected MaxRequests 100, got %d", validated.MaxRequests)
	}
}

func TestDefaultTokenBucketConfig(t *testing.T) {
	config := DefaultTokenBucketConfig()

	if config.Capacity != 100 {
		t.Errorf("expected Capacity 100, got %d", config.Capacity)
	}
	if config.RefillRate != 10.0 {
		t.Errorf("expected RefillRate 10.0, got %f", config.RefillRate)
	}
}

func TestDefaultSlidingWindowConfig(t *testing.T) {
	config := DefaultSlidingWindowConfig()

	if config.WindowSize != time.Minute {
		t.Errorf("expected WindowSize 1m, got %v", config.WindowSize)
	}
	if config.MaxRequests != 100 {
		t.Errorf("expected MaxRequests 100, got %d", config.MaxRequests)
	}
}

// =============================================================================
// Middleware Tests
// =============================================================================

func TestRateLimitInterceptor_Check(t *testing.T) {
	config := &TokenBucketConfig{Capacity: 2, RefillRate: 1.0}
	limiter := NewTokenBucket(config)
	interceptor := NewRateLimitInterceptor(limiter)
	ctx := context.Background()

	if err := interceptor.Check(ctx); err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	if err := interceptor.Check(ctx); err != nil {
		t.Errorf("expected no error, got %v", err)
	}

	// Third should fail
	if err := interceptor.Check(ctx); err == nil {
		t.Error("expected error, got nil")
	}
}

func TestRateLimitInterceptor_RetryAfter(t *testing.T) {
	config := &TokenBucketConfig{Capacity: 1, RefillRate: 10.0}
	limiter := NewTokenBucket(config)
	interceptor := NewRateLimitInterceptor(limiter)

	// Before consuming
	if d := interceptor.RetryAfter(); d != 0 {
		t.Errorf("expected 0, got %v", d)
	}

	_ = interceptor.Check(context.Background())

	if d := interceptor.RetryAfter(); d <= 0 {
		t.Errorf("expected positive duration, got %v", d)
	}
}

func TestRateLimitInterceptor_Stats(t *testing.T) {
	config := &TokenBucketConfig{Capacity: 5, RefillRate: 1.0}
	limiter := NewTokenBucket(config)
	interceptor := NewRateLimitInterceptor(limiter)

	_ = interceptor.Check(context.Background())
	_ = interceptor.Check(context.Background())

	stats := interceptor.Stats()
	if stats.Allowed != 2 {
		t.Errorf("expected 2 allowed, got %d", stats.Allowed)
	}
	if stats.Total != 2 {
		t.Errorf("expected 2 total, got %d", stats.Total)
	}
}

// =============================================================================
// Error Tests
// =============================================================================

func TestErrRateLimitExceeded_Error(t *testing.T) {
	err := &ErrRateLimitExceeded{RetryAfter: 5 * time.Second}
	expected := "rate limit exceeded, retry after 5s"
	if err.Error() != expected {
		t.Errorf("expected %q, got %q", expected, err.Error())
	}
}

func TestGetRetryAfter(t *testing.T) {
	// Rate limit error
	err := &ErrRateLimitExceeded{RetryAfter: 5 * time.Second}
	if d := GetRetryAfter(err); d != 5*time.Second {
		t.Errorf("expected 5s, got %v", d)
	}

	// Non-rate limit error
	if d := GetRetryAfter(context.Canceled); d != 0 {
		t.Errorf("expected 0, got %v", d)
	}
}

func TestStats_RejectionRate(t *testing.T) {
	stats := Stats{Allowed: 8, Rejected: 2, Total: 10}
	if rate := stats.RejectionRate(); rate != 0.2 {
		t.Errorf("expected 0.2, got %f", rate)
	}

	empty := Stats{}
	if rate := empty.RejectionRate(); rate != 0 {
		t.Errorf("expected 0, got %f", rate)
	}
}
