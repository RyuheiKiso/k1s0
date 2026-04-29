// 本ファイルは IdempotencyCache の単体テスト。
//
// 検証観点:
//   - cache hit で fn が再実行されない
//   - 異なる idempotency_key は独立に処理される
//   - 異なる tenant 間の idempotency_key は衝突しない
//   - fn error は cache されない（次回再試行で fn 再実行）
//   - 並行同 key の場合 fn は 1 回しか呼ばれない（singleflight 挙動）
//   - 空 idempotency_key の IdempotencyKey は空文字（dedup 対象外を表現）
//   - TTL 経過後は再 fn 実行

package common

import (
	"context"
	"errors"
	"sync"
	"sync/atomic"
	"testing"
	"time"
)

// 同 key 2 回呼出で fn は 1 回しか走らない。
func TestIdempotencyCache_HitDoesNotReExecute(t *testing.T) {
	c := NewInMemoryIdempotencyCache(time.Hour)
	var calls int32
	fn := func() (interface{}, error) {
		atomic.AddInt32(&calls, 1)
		return "result-1", nil
	}
	r1, err := c.GetOrCompute(context.Background(), "k", fn)
	if err != nil || r1 != "result-1" {
		t.Fatalf("first: %v %v", r1, err)
	}
	r2, err := c.GetOrCompute(context.Background(), "k", fn)
	if err != nil || r2 != "result-1" {
		t.Fatalf("second: %v %v", r2, err)
	}
	if atomic.LoadInt32(&calls) != 1 {
		t.Fatalf("fn calls = %d want 1", calls)
	}
}

// 異なる key は独立。
func TestIdempotencyCache_DifferentKeysIndependent(t *testing.T) {
	c := NewInMemoryIdempotencyCache(time.Hour)
	r1, _ := c.GetOrCompute(context.Background(), "k1", func() (interface{}, error) { return "v1", nil })
	r2, _ := c.GetOrCompute(context.Background(), "k2", func() (interface{}, error) { return "v2", nil })
	if r1 != "v1" || r2 != "v2" {
		t.Fatalf("results: %v %v", r1, r2)
	}
}

// fn error は cache されない（次回 fn 再実行）。
func TestIdempotencyCache_ErrorNotCached(t *testing.T) {
	c := NewInMemoryIdempotencyCache(time.Hour)
	var calls int32
	failed := errors.New("transient")
	fn := func() (interface{}, error) {
		n := atomic.AddInt32(&calls, 1)
		if n == 1 {
			return nil, failed
		}
		return "v-second", nil
	}
	if _, err := c.GetOrCompute(context.Background(), "k", fn); !errors.Is(err, failed) {
		t.Fatalf("first should fail: %v", err)
	}
	r, err := c.GetOrCompute(context.Background(), "k", fn)
	if err != nil || r != "v-second" {
		t.Fatalf("second should succeed: %v %v", r, err)
	}
	if calls != 2 {
		t.Errorf("calls = %d want 2", calls)
	}
}

// 並行同 key で fn は 1 回しか呼ばれない（singleflight 挙動）。
func TestIdempotencyCache_ConcurrentSameKey_Singleflight(t *testing.T) {
	c := NewInMemoryIdempotencyCache(time.Hour)
	var calls int32
	fn := func() (interface{}, error) {
		atomic.AddInt32(&calls, 1)
		// 並行性を強制するため少し sleep する。
		time.Sleep(10 * time.Millisecond)
		return "v", nil
	}
	var wg sync.WaitGroup
	for i := 0; i < 10; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			_, _ = c.GetOrCompute(context.Background(), "k", fn)
		}()
	}
	wg.Wait()
	if got := atomic.LoadInt32(&calls); got != 1 {
		t.Fatalf("calls = %d want 1 (singleflight)", got)
	}
}

// IdempotencyKey: 異なる tenant の同一 client key は別 cache key になる。
func TestIdempotencyKey_TenantIsolation(t *testing.T) {
	a := IdempotencyKey("tenant-A", "Publish", "client-key-1")
	b := IdempotencyKey("tenant-B", "Publish", "client-key-1")
	if a == b {
		t.Fatalf("tenants must produce different cache keys: %q == %q", a, b)
	}
}

// IdempotencyKey: 空 client key は空文字を返す（dedup 対象外）。
func TestIdempotencyKey_EmptyClient_ReturnsEmpty(t *testing.T) {
	if IdempotencyKey("T", "RPC", "") != "" {
		t.Fatalf("empty client key must yield empty cache key")
	}
}

// TTL 経過後は再 fn 実行。
func TestIdempotencyCache_TTLExpiry_ReExecutes(t *testing.T) {
	c := NewInMemoryIdempotencyCache(50 * time.Millisecond)
	var calls int32
	fn := func() (interface{}, error) {
		atomic.AddInt32(&calls, 1)
		return "v", nil
	}
	_, _ = c.GetOrCompute(context.Background(), "k", fn)
	// TTL 経過を待つ。
	time.Sleep(80 * time.Millisecond)
	_, _ = c.GetOrCompute(context.Background(), "k", fn)
	if calls != 2 {
		t.Fatalf("calls = %d want 2 (after TTL expiry)", calls)
	}
}
