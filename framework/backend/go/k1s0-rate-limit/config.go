package k1s0ratelimit

import "time"

// TokenBucketConfig holds configuration for the token bucket rate limiter.
type TokenBucketConfig struct {
	// Capacity is the maximum number of tokens in the bucket.
	// Must be at least 1. Default is 100.
	Capacity int64 `yaml:"capacity"`

	// RefillRate is the number of tokens added per second.
	// Must be positive. Default is 10.0.
	RefillRate float64 `yaml:"refill_rate"`
}

// DefaultTokenBucketConfig returns a TokenBucketConfig with default values.
func DefaultTokenBucketConfig() *TokenBucketConfig {
	return &TokenBucketConfig{
		Capacity:   100,
		RefillRate: 10.0,
	}
}

// Validate validates the configuration and applies defaults.
func (c *TokenBucketConfig) Validate() *TokenBucketConfig {
	if c.Capacity < 1 {
		c.Capacity = 100
	}
	if c.RefillRate <= 0 {
		c.RefillRate = 10.0
	}
	return c
}

// SlidingWindowConfig holds configuration for the sliding window rate limiter.
type SlidingWindowConfig struct {
	// WindowSize is the time window for counting requests.
	// Must be positive. Default is 1 minute.
	WindowSize time.Duration `yaml:"window_size"`

	// MaxRequests is the maximum number of requests allowed within the window.
	// Must be at least 1. Default is 100.
	MaxRequests int64 `yaml:"max_requests"`
}

// DefaultSlidingWindowConfig returns a SlidingWindowConfig with default values.
func DefaultSlidingWindowConfig() *SlidingWindowConfig {
	return &SlidingWindowConfig{
		WindowSize:  time.Minute,
		MaxRequests: 100,
	}
}

// Validate validates the configuration and applies defaults.
func (c *SlidingWindowConfig) Validate() *SlidingWindowConfig {
	if c.WindowSize <= 0 {
		c.WindowSize = time.Minute
	}
	if c.MaxRequests < 1 {
		c.MaxRequests = 100
	}
	return c
}
