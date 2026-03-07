package bulkhead_test

import (
	"context"
	"errors"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/k1s0-platform/system-library-go-bulkhead"
	"github.com/stretchr/testify/assert"
)

func defaultConfig() bulkhead.Config {
	return bulkhead.Config{
		MaxConcurrentCalls: 2,
		MaxWaitDuration:    50 * time.Millisecond,
	}
}

func TestAcquire_Release(t *testing.T) {
	bh := bulkhead.New(defaultConfig())
	ctx := context.Background()

	// スロットを取得できることを確認
	err := bh.Acquire(ctx)
	assert.NoError(t, err)

	// 2つ目のスロットも取得できることを確認
	err = bh.Acquire(ctx)
	assert.NoError(t, err)

	// 解放後に再度取得できることを確認
	bh.Release()
	err = bh.Acquire(ctx)
	assert.NoError(t, err)

	// クリーンアップ
	bh.Release()
	bh.Release()
}

func TestFull_Rejection(t *testing.T) {
	bh := bulkhead.New(defaultConfig())
	ctx := context.Background()

	// 全スロットを取得
	err := bh.Acquire(ctx)
	assert.NoError(t, err)
	err = bh.Acquire(ctx)
	assert.NoError(t, err)

	// 3つ目はタイムアウトで拒否されることを確認
	err = bh.Acquire(ctx)
	assert.ErrorIs(t, err, bulkhead.ErrFull)

	// クリーンアップ
	bh.Release()
	bh.Release()
}

func TestWait_Timeout(t *testing.T) {
	bh := bulkhead.New(defaultConfig())
	ctx := context.Background()

	// 全スロットを取得
	_ = bh.Acquire(ctx)
	_ = bh.Acquire(ctx)

	// MaxWaitDuration 後に ErrFull が返ることを確認
	start := time.Now()
	err := bh.Acquire(ctx)
	elapsed := time.Since(start)

	assert.ErrorIs(t, err, bulkhead.ErrFull)
	assert.GreaterOrEqual(t, elapsed, 50*time.Millisecond)

	// クリーンアップ
	bh.Release()
	bh.Release()
}

func TestContext_Cancellation(t *testing.T) {
	bh := bulkhead.New(defaultConfig())
	ctx, cancel := context.WithCancel(context.Background())

	// 全スロットを取得
	_ = bh.Acquire(context.Background())
	_ = bh.Acquire(context.Background())

	// コンテキストをキャンセルして中断されることを確認
	go func() {
		time.Sleep(10 * time.Millisecond)
		cancel()
	}()

	err := bh.Acquire(ctx)
	assert.ErrorIs(t, err, context.Canceled)

	// クリーンアップ
	bh.Release()
	bh.Release()
}

func TestConcurrent_Access(t *testing.T) {
	cfg := defaultConfig()
	bh := bulkhead.New(cfg)
	ctx := context.Background()

	var currentConcurrency atomic.Int32
	var maxConcurrency atomic.Int32
	var wg sync.WaitGroup

	for i := 0; i < 10; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			err := bh.Acquire(ctx)
			if err != nil {
				return
			}
			defer bh.Release()

			cur := currentConcurrency.Add(1)
			// 最大同時実行数を記録
			for {
				old := maxConcurrency.Load()
				if cur <= old || maxConcurrency.CompareAndSwap(old, cur) {
					break
				}
			}

			time.Sleep(20 * time.Millisecond)
			currentConcurrency.Add(-1)
		}()
	}

	wg.Wait()

	// 同時実行が MaxConcurrentCalls を超えないことを確認
	assert.LessOrEqual(t, maxConcurrency.Load(), int32(cfg.MaxConcurrentCalls))
}

func TestCall_Success(t *testing.T) {
	bh := bulkhead.New(defaultConfig())
	ctx := context.Background()

	called := false
	err := bh.Call(ctx, func() error {
		called = true
		return nil
	})

	assert.NoError(t, err)
	assert.True(t, called)
}

func TestCall_Full(t *testing.T) {
	bh := bulkhead.New(defaultConfig())
	ctx := context.Background()

	// 全スロットを取得
	_ = bh.Acquire(ctx)
	_ = bh.Acquire(ctx)

	// Call が満杯時に ErrFull を返すことを確認
	err := bh.Call(ctx, func() error {
		return errors.New("実行されないはず")
	})
	assert.ErrorIs(t, err, bulkhead.ErrFull)

	// クリーンアップ
	bh.Release()
	bh.Release()
}
