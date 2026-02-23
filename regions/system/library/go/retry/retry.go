package retry

import (
	"context"
	"fmt"
	"math"
	"math/rand"
	"time"
)

// RetryConfig はリトライポリシーの設定。
type RetryConfig struct {
	// MaxAttempts は最大試行回数。
	MaxAttempts int
	// InitialDelay は初回リトライの待機時間。
	InitialDelay time.Duration
	// MaxDelay は最大待機時間。
	MaxDelay time.Duration
	// Multiplier は指数バックオフの倍率。
	Multiplier float64
	// Jitter はジッター（揺らぎ）を有効にするかどうか。
	Jitter bool
}

// DefaultRetryConfig はデフォルトのリトライ設定を返す。
func DefaultRetryConfig() *RetryConfig {
	return &RetryConfig{
		MaxAttempts:  3,
		InitialDelay: 100 * time.Millisecond,
		MaxDelay:     30 * time.Second,
		Multiplier:   2.0,
		Jitter:       true,
	}
}

// ComputeDelay は指定された試行回数に対する待機時間を計算する。
func (c *RetryConfig) ComputeDelay(attempt int) time.Duration {
	base := float64(c.InitialDelay.Milliseconds()) * math.Pow(c.Multiplier, float64(attempt))
	capped := math.Min(base, float64(c.MaxDelay.Milliseconds()))
	if c.Jitter {
		jitterRange := capped * 0.1
		capped = capped - jitterRange + rand.Float64()*jitterRange*2.0
	}
	return time.Duration(capped) * time.Millisecond
}

// RetryError はリトライ失敗を表すエラー。
type RetryError struct {
	// Attempts は試行回数。
	Attempts int
	// LastError は最後のエラー。
	LastError error
}

func (e *RetryError) Error() string {
	return fmt.Sprintf("すべてのリトライが失敗しました (%d 回): %v", e.Attempts, e.LastError)
}

func (e *RetryError) Unwrap() error {
	return e.LastError
}

// WithRetry はリトライポリシーに従って操作を実行する。
func WithRetry[T any](ctx context.Context, config *RetryConfig, operation func(ctx context.Context) (T, error)) (T, error) {
	var lastErr error
	var zero T
	for attempt := 0; attempt < config.MaxAttempts; attempt++ {
		result, err := operation(ctx)
		if err == nil {
			return result, nil
		}
		lastErr = err
		if attempt+1 < config.MaxAttempts {
			delay := config.ComputeDelay(attempt)
			select {
			case <-ctx.Done():
				return zero, ctx.Err()
			case <-time.After(delay):
			}
		}
	}
	return zero, &RetryError{Attempts: config.MaxAttempts, LastError: lastErr}
}
