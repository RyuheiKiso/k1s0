// Package k1s0resilience provides resilience patterns for the k1s0 framework.
//
// This package implements common resilience patterns:
//   - Retry with exponential backoff and jitter
//   - Circuit breaker for failure isolation
//   - Timeout for deadline enforcement
//   - Bulkhead for concurrency limiting
//
// # Usage
//
//	// Retry with exponential backoff
//	result, err := k1s0resilience.Retry(ctx, retryConfig, func() (string, error) {
//	    return apiClient.Call()
//	})
//
//	// Circuit breaker
//	cb := k1s0resilience.NewCircuitBreaker(cbConfig)
//	result, err := cb.Execute(func() (interface{}, error) {
//	    return apiClient.Call()
//	})
//
//	// Timeout
//	result, err := k1s0resilience.WithTimeout(ctx, 5*time.Second, func(ctx context.Context) (string, error) {
//	    return apiClient.Call(ctx)
//	})
//
//	// Bulkhead (concurrency limiting)
//	bh := k1s0resilience.NewBulkhead(10)
//	result, err := bh.Execute(ctx, func() (string, error) {
//	    return apiClient.Call()
//	})
package k1s0resilience

import (
	"time"
)

// RetryConfig holds configuration for retry operations.
type RetryConfig struct {
	// MaxAttempts is the maximum number of retry attempts (including the initial call).
	// Must be at least 1. Default is 3.
	MaxAttempts int

	// InitialInterval is the initial backoff interval.
	// Default is 100ms.
	InitialInterval time.Duration

	// MaxInterval is the maximum backoff interval.
	// Default is 10s.
	MaxInterval time.Duration

	// Multiplier is the exponential backoff multiplier.
	// Default is 2.0.
	Multiplier float64

	// JitterFactor is the jitter factor (0.0 to 1.0).
	// Default is 0.1 (10% jitter).
	JitterFactor float64

	// RetryableChecker is a function that determines if an error is retryable.
	// If nil, all errors are retryable.
	RetryableChecker func(error) bool
}

// DefaultRetryConfig returns a RetryConfig with default values.
func DefaultRetryConfig() *RetryConfig {
	return &RetryConfig{
		MaxAttempts:     3,
		InitialInterval: 100 * time.Millisecond,
		MaxInterval:     10 * time.Second,
		Multiplier:      2.0,
		JitterFactor:    0.1,
	}
}

// Validate validates the retry configuration and applies defaults.
func (c *RetryConfig) Validate() *RetryConfig {
	if c.MaxAttempts < 1 {
		c.MaxAttempts = 3
	}
	if c.InitialInterval <= 0 {
		c.InitialInterval = 100 * time.Millisecond
	}
	if c.MaxInterval <= 0 {
		c.MaxInterval = 10 * time.Second
	}
	if c.Multiplier <= 0 {
		c.Multiplier = 2.0
	}
	if c.JitterFactor < 0 || c.JitterFactor > 1 {
		c.JitterFactor = 0.1
	}
	return c
}

// CircuitBreakerConfig holds configuration for the circuit breaker.
type CircuitBreakerConfig struct {
	// Name is the name of the circuit breaker (for logging).
	Name string

	// MaxRequests is the maximum number of requests allowed in half-open state.
	// Default is 1.
	MaxRequests uint32

	// Interval is the cyclic period of the closed state for clearing counts.
	// If 0, the counts are never cleared.
	Interval time.Duration

	// Timeout is the period of the open state before transitioning to half-open.
	// Default is 60s.
	Timeout time.Duration

	// FailureThreshold is the number of failures required to open the circuit.
	// Default is 5.
	FailureThreshold uint32

	// SuccessThreshold is the number of successes required to close the circuit.
	// Default is 1.
	SuccessThreshold uint32

	// FailureRatio is the failure ratio threshold (0.0 to 1.0).
	// If set, the circuit opens when the failure ratio exceeds this value.
	// Overrides FailureThreshold if both are set.
	FailureRatio float64

	// MinRequestsForRatio is the minimum number of requests required before
	// the failure ratio is calculated.
	// Default is 10.
	MinRequestsForRatio uint32

	// OnStateChange is called when the circuit breaker state changes.
	OnStateChange func(name string, from, to State)
}

// DefaultCircuitBreakerConfig returns a CircuitBreakerConfig with default values.
func DefaultCircuitBreakerConfig(name string) *CircuitBreakerConfig {
	return &CircuitBreakerConfig{
		Name:                name,
		MaxRequests:         1,
		Interval:            0,
		Timeout:             60 * time.Second,
		FailureThreshold:    5,
		SuccessThreshold:    1,
		FailureRatio:        0,
		MinRequestsForRatio: 10,
	}
}

// Validate validates the circuit breaker configuration and applies defaults.
func (c *CircuitBreakerConfig) Validate() *CircuitBreakerConfig {
	if c.Name == "" {
		c.Name = "default"
	}
	if c.MaxRequests == 0 {
		c.MaxRequests = 1
	}
	if c.Timeout <= 0 {
		c.Timeout = 60 * time.Second
	}
	if c.FailureThreshold == 0 {
		c.FailureThreshold = 5
	}
	if c.SuccessThreshold == 0 {
		c.SuccessThreshold = 1
	}
	if c.MinRequestsForRatio == 0 {
		c.MinRequestsForRatio = 10
	}
	return c
}

// BulkheadConfig holds configuration for the bulkhead.
type BulkheadConfig struct {
	// MaxConcurrent is the maximum number of concurrent executions.
	// Must be at least 1. Default is 10.
	MaxConcurrent int

	// MaxWaitTime is the maximum time to wait for a slot.
	// If 0, the call returns immediately if no slot is available.
	// Default is 0 (no waiting).
	MaxWaitTime time.Duration
}

// DefaultBulkheadConfig returns a BulkheadConfig with default values.
func DefaultBulkheadConfig() *BulkheadConfig {
	return &BulkheadConfig{
		MaxConcurrent: 10,
		MaxWaitTime:   0,
	}
}

// Validate validates the bulkhead configuration and applies defaults.
func (c *BulkheadConfig) Validate() *BulkheadConfig {
	if c.MaxConcurrent < 1 {
		c.MaxConcurrent = 10
	}
	return c
}
