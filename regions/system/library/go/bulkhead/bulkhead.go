package bulkhead

import (
	"context"
	"errors"
	"time"
)

// ErrFull はバルクヘッドが満杯で待機タイムアウトのエラー。
var ErrFull = errors.New("バルクヘッドが満杯です")

// Config はバルクヘッドの設定。
type Config struct {
	MaxConcurrentCalls int
	MaxWaitDuration    time.Duration
}

// Bulkhead はバルクヘッド。
type Bulkhead struct {
	config Config
	sem    chan struct{}
}

// New は新しい Bulkhead を生成する。
func New(cfg Config) *Bulkhead {
	sem := make(chan struct{}, cfg.MaxConcurrentCalls)
	for i := 0; i < cfg.MaxConcurrentCalls; i++ {
		sem <- struct{}{}
	}
	return &Bulkhead{
		config: cfg,
		sem:    sem,
	}
}

// Acquire はバルクヘッドのスロットを取得する。満杯の場合は MaxWaitDuration まで待機する。
func (b *Bulkhead) Acquire(ctx context.Context) error {
	timer := time.NewTimer(b.config.MaxWaitDuration)
	defer timer.Stop()

	select {
	case <-b.sem:
		return nil
	case <-ctx.Done():
		return ctx.Err()
	case <-timer.C:
		return ErrFull
	}
}

// Release はバルクヘッドのスロットを解放する。
func (b *Bulkhead) Release() {
	b.sem <- struct{}{}
}

// Call は関数をバルクヘッドで保護して実行する。
func (b *Bulkhead) Call(ctx context.Context, fn func() error) error {
	if err := b.Acquire(ctx); err != nil {
		return err
	}
	defer b.Release()
	return fn()
}
