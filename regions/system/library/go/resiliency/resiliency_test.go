package resiliency

import (
	"context"
	"errors"
	"fmt"
	"sync/atomic"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestExecute_Success(t *testing.T) {
	policy := ResiliencyPolicy{}
	dec := NewResiliencyDecorator(policy)

	result, err := Execute[int](context.Background(), dec, func() (int, error) {
		return 42, nil
	})

	require.NoError(t, err)
	assert.Equal(t, 42, result)
}

func TestExecute_RetrySuccess(t *testing.T) {
	policy := ResiliencyPolicy{
		Retry: &RetryConfig{
			MaxAttempts: 3,
			BaseDelay:   10 * time.Millisecond,
			MaxDelay:    100 * time.Millisecond,
		},
	}
	dec := NewResiliencyDecorator(policy)

	var counter atomic.Int32
	result, err := Execute[int](context.Background(), dec, func() (int, error) {
		c := counter.Add(1)
		if c < 3 {
			return 0, errors.New("fail")
		}
		return 99, nil
	})

	require.NoError(t, err)
	assert.Equal(t, 99, result)
	assert.Equal(t, int32(3), counter.Load())
}

func TestExecute_MaxRetriesExceeded(t *testing.T) {
	policy := ResiliencyPolicy{
		Retry: &RetryConfig{
			MaxAttempts: 2,
			BaseDelay:   1 * time.Millisecond,
			MaxDelay:    10 * time.Millisecond,
		},
	}
	dec := NewResiliencyDecorator(policy)

	_, err := Execute[int](context.Background(), dec, func() (int, error) {
		return 0, errors.New("always fail")
	})

	require.Error(t, err)
	var rErr *ResiliencyError
	require.True(t, errors.As(err, &rErr))
	assert.Equal(t, "max_retries_exceeded", rErr.Kind)
}

func TestExecute_Timeout(t *testing.T) {
	policy := ResiliencyPolicy{
		Timeout: 50 * time.Millisecond,
	}
	dec := NewResiliencyDecorator(policy)

	_, err := Execute[int](context.Background(), dec, func() (int, error) {
		time.Sleep(1 * time.Second)
		return 42, nil
	})

	require.Error(t, err)
	var rErr *ResiliencyError
	require.True(t, errors.As(err, &rErr))
	assert.Equal(t, "timeout", rErr.Kind)
}

func TestExecute_CircuitBreakerOpens(t *testing.T) {
	policy := ResiliencyPolicy{
		CircuitBreaker: &CircuitBreakerConfig{
			FailureThreshold: 3,
			RecoveryTimeout:  1 * time.Minute,
			HalfOpenMaxCalls: 1,
		},
	}
	dec := NewResiliencyDecorator(policy)

	// Trip the circuit breaker
	for i := 0; i < 3; i++ {
		_, _ = Execute[int](context.Background(), dec, func() (int, error) {
			return 0, errors.New("fail")
		})
	}

	// Next call should fail with circuit open
	_, err := Execute[int](context.Background(), dec, func() (int, error) {
		return 42, nil
	})

	require.Error(t, err)
	var rErr *ResiliencyError
	require.True(t, errors.As(err, &rErr))
	assert.Equal(t, "circuit_open", rErr.Kind)
}

func TestExecute_BulkheadFull(t *testing.T) {
	policy := ResiliencyPolicy{
		Bulkhead: &BulkheadConfig{
			MaxConcurrentCalls: 1,
			MaxWaitDuration:    50 * time.Millisecond,
		},
	}
	dec := NewResiliencyDecorator(policy)

	started := make(chan struct{})
	done := make(chan struct{})

	// Occupy the single bulkhead slot
	go func() {
		_, _ = Execute[int](context.Background(), dec, func() (int, error) {
			close(started)
			<-done
			return 1, nil
		})
	}()

	<-started

	// This should fail with bulkhead full
	_, err := Execute[int](context.Background(), dec, func() (int, error) {
		return 2, nil
	})

	close(done)

	require.Error(t, err)
	var rErr *ResiliencyError
	require.True(t, errors.As(err, &rErr))
	assert.Equal(t, "bulkhead_full", rErr.Kind)
}

func TestCalculateBackoff(t *testing.T) {
	base := 100 * time.Millisecond
	max := 5 * time.Second

	assert.Equal(t, 100*time.Millisecond, calculateBackoff(0, base, max))
	assert.Equal(t, 200*time.Millisecond, calculateBackoff(1, base, max))
	assert.Equal(t, 400*time.Millisecond, calculateBackoff(2, base, max))

	// Should cap at max
	result := calculateBackoff(20, base, max)
	assert.True(t, result <= max, fmt.Sprintf("expected <= %v, got %v", max, result))
}
