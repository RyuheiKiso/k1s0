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

// WithRetryが初回試行で成功した場合にリトライなしで結果を返すことを確認する。
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

// WithRetryが2回失敗した後3回目の試行で成功することを確認する。
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

// 全リトライが失敗した場合にRetryErrorと試行回数が正しく返ることを確認する。
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

// コンテキストがキャンセルされた場合にWithRetryがリトライを中断してエラーを返すことを確認する。
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

// ComputeDelayが指数バックオフで遅延時間を正しく計算することを確認する。
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

// ジッターなし設定時にComputeDelayが同一試行番号で常に同じ遅延を返すことを確認する。
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

// サーキットブレーカーが失敗閾値に達するとオープン状態になることを確認する。
func TestCircuitBreaker_OpensAfterThreshold(t *testing.T) {
	cb := retry.NewCircuitBreaker(&retry.CircuitBreakerConfig{
		FailureThreshold: 3,
		HalfOpenSuccess:  2,
		OpenTimeout:      30 * time.Second,
	})

	assert.False(t, cb.IsOpen())
	cb.RecordFailure()
	cb.RecordFailure()
	assert.False(t, cb.IsOpen())
	cb.RecordFailure()
	assert.True(t, cb.IsOpen())
}

// オープンタイムアウト経過後にサーキットブレーカーがハーフオープン状態に遷移することを確認する。
func TestCircuitBreaker_TransitionsToHalfOpen(t *testing.T) {
	cb := retry.NewCircuitBreaker(&retry.CircuitBreakerConfig{
		FailureThreshold: 2,
		HalfOpenSuccess:  1,
		OpenTimeout:      50 * time.Millisecond,
	})

	cb.RecordFailure()
	cb.RecordFailure()
	assert.True(t, cb.IsOpen())

	time.Sleep(60 * time.Millisecond)
	assert.False(t, cb.IsOpen())
	assert.Equal(t, retry.StateHalfOpen, cb.GetState())
}

// ハーフオープン状態で必要な成功回数を記録するとサーキットブレーカーがクローズ状態に戻ることを確認する。
func TestCircuitBreaker_ClosesAfterSuccesses(t *testing.T) {
	cb := retry.NewCircuitBreaker(&retry.CircuitBreakerConfig{
		FailureThreshold: 2,
		HalfOpenSuccess:  2,
		OpenTimeout:      50 * time.Millisecond,
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
