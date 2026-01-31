package k1s0db

import (
	"context"
	"sync"
	"testing"
	"time"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/jackc/pgx/v5/pgxpool"
)

// mockPool implements the Pool interface for testing.
type mockPool struct {
	pingDelay time.Duration
	pingErr   error
}

func (m *mockPool) Exec(_ context.Context, _ string, _ ...any) (pgconn.CommandTag, error) {
	return pgconn.CommandTag{}, nil
}
func (m *mockPool) Query(_ context.Context, _ string, _ ...any) (pgx.Rows, error) {
	return nil, nil
}
func (m *mockPool) QueryRow(_ context.Context, _ string, _ ...any) pgx.Row {
	return nil
}
func (m *mockPool) Begin(_ context.Context) (pgx.Tx, error) {
	return nil, nil
}
func (m *mockPool) BeginTx(_ context.Context, _ pgx.TxOptions) (pgx.Tx, error) {
	return nil, nil
}
func (m *mockPool) Ping(ctx context.Context) error {
	if m.pingDelay > 0 {
		select {
		case <-time.After(m.pingDelay):
		case <-ctx.Done():
			return ctx.Err()
		}
	}
	return m.pingErr
}
func (m *mockPool) Close() {}
func (m *mockPool) Stats() *pgxpool.Stat {
	return &pgxpool.Stat{}
}

func TestBackpressuredPool_Acquire(t *testing.T) {
	pool := &mockPool{}
	config := DefaultBackpressuredPoolConfig()
	bp := NewBackpressuredPool(pool, *config)

	err := bp.Acquire(context.Background())
	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
}

func TestBackpressuredPool_ExceedsMaxWaiting(t *testing.T) {
	pool := &mockPool{pingDelay: 200 * time.Millisecond}
	config := &BackpressuredPoolConfig{
		MaxWaiting:     2,
		AcquireTimeout: time.Second,
	}
	bp := NewBackpressuredPool(pool, *config)

	var wg sync.WaitGroup
	rejected := 0
	var mu sync.Mutex

	// Start 5 goroutines but only 2 should wait
	for i := 0; i < 5; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			err := bp.Acquire(context.Background())
			if err == ErrPoolBackpressure {
				mu.Lock()
				rejected++
				mu.Unlock()
			}
		}()
		time.Sleep(5 * time.Millisecond) // stagger slightly
	}

	wg.Wait()

	if rejected == 0 {
		t.Error("expected some rejections due to max waiting")
	}
}

func TestBackpressuredPool_AcquireTimeout(t *testing.T) {
	pool := &mockPool{pingDelay: time.Second}
	config := &BackpressuredPoolConfig{
		MaxWaiting:     100,
		AcquireTimeout: 50 * time.Millisecond,
	}
	bp := NewBackpressuredPool(pool, *config)

	err := bp.Acquire(context.Background())
	if err == nil {
		t.Error("expected error due to timeout")
	}
}

func TestBackpressuredPool_ContextCancelled(t *testing.T) {
	pool := &mockPool{}
	config := DefaultBackpressuredPoolConfig()
	bp := NewBackpressuredPool(pool, *config)

	ctx, cancel := context.WithCancel(context.Background())
	cancel()

	err := bp.Acquire(ctx)
	if err == nil {
		t.Error("expected error for cancelled context")
	}
}

func TestBackpressuredPool_Stats(t *testing.T) {
	pool := &mockPool{}
	config := DefaultBackpressuredPoolConfig()
	bp := NewBackpressuredPool(pool, *config)

	stats := bp.Stats()
	if stats.WaitingCount != 0 {
		t.Errorf("expected 0 waiting, got %d", stats.WaitingCount)
	}
	if stats.RejectedTotal != 0 {
		t.Errorf("expected 0 rejected, got %d", stats.RejectedTotal)
	}
}

func TestBackpressuredPool_Pool(t *testing.T) {
	pool := &mockPool{}
	config := DefaultBackpressuredPoolConfig()
	bp := NewBackpressuredPool(pool, *config)

	if bp.Pool() != pool {
		t.Error("expected same pool reference")
	}
}

func TestBackpressuredPoolConfig_Validate(t *testing.T) {
	config := &BackpressuredPoolConfig{MaxWaiting: -1, AcquireTimeout: -1}
	validated := config.Validate()

	if validated.MaxWaiting != 100 {
		t.Errorf("expected 100, got %d", validated.MaxWaiting)
	}
	if validated.AcquireTimeout != 5*time.Second {
		t.Errorf("expected 5s, got %v", validated.AcquireTimeout)
	}
}

func TestDefaultBackpressuredPoolConfig(t *testing.T) {
	config := DefaultBackpressuredPoolConfig()

	if config.MaxWaiting != 100 {
		t.Errorf("expected 100, got %d", config.MaxWaiting)
	}
	if config.AcquireTimeout != 5*time.Second {
		t.Errorf("expected 5s, got %v", config.AcquireTimeout)
	}
}
