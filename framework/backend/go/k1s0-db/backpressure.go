package k1s0db

import (
	"context"
	"errors"
	"sync/atomic"
	"time"
)

// ErrPoolBackpressure is returned when too many goroutines are waiting for a connection.
var ErrPoolBackpressure = errors.New("pool backpressure: too many waiters")

// BackpressuredPoolConfig holds configuration for pool backpressure.
type BackpressuredPoolConfig struct {
	// MaxWaiting is the maximum number of goroutines allowed to wait for a connection.
	// Must be at least 1. Default is 100.
	MaxWaiting int `yaml:"max_waiting"`

	// AcquireTimeout is the maximum time to wait for a connection.
	// Default is 5 seconds.
	AcquireTimeout time.Duration `yaml:"acquire_timeout"`
}

// DefaultBackpressuredPoolConfig returns a BackpressuredPoolConfig with default values.
func DefaultBackpressuredPoolConfig() *BackpressuredPoolConfig {
	return &BackpressuredPoolConfig{
		MaxWaiting:     100,
		AcquireTimeout: 5 * time.Second,
	}
}

// Validate validates the configuration and applies defaults.
func (c *BackpressuredPoolConfig) Validate() *BackpressuredPoolConfig {
	if c.MaxWaiting < 1 {
		c.MaxWaiting = 100
	}
	if c.AcquireTimeout <= 0 {
		c.AcquireTimeout = 5 * time.Second
	}
	return c
}

// PoolBackpressureMetrics holds pool backpressure metrics.
type PoolBackpressureMetrics struct {
	// ActiveConnections is the number of currently acquired connections.
	ActiveConnections int64

	// WaitingCount is the number of goroutines currently waiting for a connection.
	WaitingCount int64

	// RejectedTotal is the total number of rejected acquire attempts.
	RejectedTotal int64
}

// BackpressuredPool wraps a Pool with backpressure to limit waiting goroutines.
//
// When the number of goroutines waiting for a connection exceeds MaxWaiting,
// new acquire attempts are immediately rejected to prevent resource exhaustion.
//
// Example:
//
//	pool, _ := k1s0db.NewConnectionPool(ctx, dbConfig)
//	bpConfig := k1s0db.DefaultBackpressuredPoolConfig()
//	bp := k1s0db.NewBackpressuredPool(pool, *bpConfig)
//
//	if err := bp.Acquire(ctx); err != nil {
//	    // Too many waiters or timeout
//	}
type BackpressuredPool struct {
	inner    Pool
	config   BackpressuredPoolConfig
	waiting  atomic.Int32
	rejected int64
}

// NewBackpressuredPool creates a new BackpressuredPool wrapping the given pool.
func NewBackpressuredPool(inner Pool, config BackpressuredPoolConfig) *BackpressuredPool {
	config.Validate()
	return &BackpressuredPool{
		inner:  inner,
		config: config,
	}
}

// Acquire checks backpressure limits and pings the pool to verify a connection can be obtained.
// Returns nil if a connection is available, error if rejected or timed out.
func (p *BackpressuredPool) Acquire(ctx context.Context) error {
	if ctx.Err() != nil {
		return ctx.Err()
	}

	// Check waiting count
	current := p.waiting.Add(1)
	if int(current) > p.config.MaxWaiting {
		p.waiting.Add(-1)
		atomic.AddInt64(&p.rejected, 1)
		return ErrPoolBackpressure
	}
	defer p.waiting.Add(-1)

	// Apply timeout
	acquireCtx, cancel := context.WithTimeout(ctx, p.config.AcquireTimeout)
	defer cancel()

	// Use Ping as a proxy for connection availability
	if err := p.inner.Ping(acquireCtx); err != nil {
		return err
	}

	return nil
}

// Stats returns the current pool backpressure metrics.
func (p *BackpressuredPool) Stats() PoolBackpressureMetrics {
	var active int64
	func() {
		defer func() {
			if r := recover(); r != nil {
				active = 0
			}
		}()
		if stat := p.inner.Stats(); stat != nil {
			active = int64(stat.AcquiredConns())
		}
	}()
	return PoolBackpressureMetrics{
		ActiveConnections: active,
		WaitingCount:      int64(p.waiting.Load()),
		RejectedTotal:     atomic.LoadInt64(&p.rejected),
	}
}

// Pool returns the underlying pool.
func (p *BackpressuredPool) Pool() Pool {
	return p.inner
}
