package retry_test

import (
	"context"
	"fmt"
	"sync/atomic"
	"testing"
	"time"

	"github.com/k1s0-platform/system-library-go-retry"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestWithRetry_SucceedsOnFirstAttempt(t *testing.T) {
	config := &retry.RetryConfig{
		MaxAttempts:  3,
		InitialDelay: time.Millisecond,
		MaxDelay:     time.Second,
		Multiplier:   2.0,
		Jitter:       false,
	}
	result, err := retry.WithRetry(context.Background(), config, func(ctx context.Context) (string, error) {
		return "success", nil
	})
	require.NoError(t, err)
	assert.Equal(t, "success", result)
}

func TestWithRetry_SucceedsOnThirdAttempt(t *testing.T) {
	var counter atomic.Int32
	config := &retry.RetryConfig{
		MaxAttempts:  3,
		InitialDelay: time.Millisecond,
		MaxDelay:     time.Second,
		Multiplier:   2.0,
		Jitter:       false,
	}
	result, err := retry.WithRetry(context.Background(), config, func(ctx context.Context) (string, error) {
		attempt := counter.Add(1)
		if attempt < 3 {
			return "", fmt.Errorf("not yet")
		}
		return "success", nil
	})
	require.NoError(t, err)
	assert.Equal(t, "success", result)
	assert.Equal(t, int32(3), counter.Load())
}

func TestWithRetry_Exhausted(t *testing.T) {
	config := &retry.RetryConfig{
		MaxAttempts:  3,
		InitialDelay: time.Millisecond,
		MaxDelay:     time.Second,
		Multiplier:   2.0,
		Jitter:       false,
	}
	_, err := retry.WithRetry(context.Background(), config, func(ctx context.Context) (string, error) {
		return "", fmt.Errorf("always fails")
	})
	require.Error(t, err)
	var retryErr *retry.RetryError
	require.ErrorAs(t, err, &retryErr)
	assert.Equal(t, 3, retryErr.Attempts)
	assert.Contains(t, retryErr.LastError.Error(), "always fails")
}

func TestWithRetry_ContextCancelled(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	var counter atomic.Int32
	config := &retry.RetryConfig{
		MaxAttempts:  5,
		InitialDelay: 100 * time.Millisecond,
		MaxDelay:     time.Second,
		Multiplier:   2.0,
		Jitter:       false,
	}
	go func() {
		time.Sleep(50 * time.Millisecond)
		cancel()
	}()
	_, err := retry.WithRetry(ctx, config, func(ctx context.Context) (string, error) {
		counter.Add(1)
		return "", fmt.Errorf("fail")
	})
	require.Error(t, err)
	assert.ErrorIs(t, err, context.Canceled)
}

func TestComputeDelay_Exponential(t *testing.T) {
	config := &retry.RetryConfig{
		MaxAttempts:  5,
		InitialDelay: 100 * time.Millisecond,
		MaxDelay:     30 * time.Second,
		Multiplier:   2.0,
		Jitter:       false,
	}
	assert.Equal(t, 100*time.Millisecond, config.ComputeDelay(0))
	assert.Equal(t, 200*time.Millisecond, config.ComputeDelay(1))
	assert.Equal(t, 400*time.Millisecond, config.ComputeDelay(2))
}

func TestComputeDelay_NoJitter(t *testing.T) {
	config := &retry.RetryConfig{
		MaxAttempts:  3,
		InitialDelay: 50 * time.Millisecond,
		MaxDelay:     30 * time.Second,
		Multiplier:   3.0,
		Jitter:       false,
	}
	d1 := config.ComputeDelay(1)
	d2 := config.ComputeDelay(1)
	assert.Equal(t, d1, d2)
	assert.Equal(t, 150*time.Millisecond, d1)
}

func TestCircuitBreaker_OpensAfterThreshold(t *testing.T) {
	cb := retry.NewCircuitBreaker(&retry.CircuitBreakerConfig{
		FailureThreshold: 3,
		SuccessThreshold: 2,
		Timeout:          30 * time.Second,
	})

	assert.False(t, cb.IsOpen())
	cb.RecordFailure()
	cb.RecordFailure()
	assert.False(t, cb.IsOpen())
	cb.RecordFailure()
	assert.True(t, cb.IsOpen())
}

func TestCircuitBreaker_TransitionsToHalfOpen(t *testing.T) {
	cb := retry.NewCircuitBreaker(&retry.CircuitBreakerConfig{
		FailureThreshold: 2,
		SuccessThreshold: 1,
		Timeout:          50 * time.Millisecond,
	})

	cb.RecordFailure()
	cb.RecordFailure()
	assert.True(t, cb.IsOpen())

	time.Sleep(60 * time.Millisecond)
	assert.False(t, cb.IsOpen())
	assert.Equal(t, retry.StateHalfOpen, cb.GetState())
}

func TestCircuitBreaker_ClosesAfterSuccesses(t *testing.T) {
	cb := retry.NewCircuitBreaker(&retry.CircuitBreakerConfig{
		FailureThreshold: 2,
		SuccessThreshold: 2,
		Timeout:          50 * time.Millisecond,
	})

	cb.RecordFailure()
	cb.RecordFailure()
	assert.True(t, cb.IsOpen())

	time.Sleep(60 * time.Millisecond)
	assert.False(t, cb.IsOpen())
	assert.Equal(t, retry.StateHalfOpen, cb.GetState())

	cb.RecordSuccess()
	assert.Equal(t, retry.StateHalfOpen, cb.GetState())
	cb.RecordSuccess()
	assert.Equal(t, retry.StateClosed, cb.GetState())
}
