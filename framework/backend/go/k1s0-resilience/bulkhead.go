package k1s0resilience

import (
	"context"
	"errors"
	"sync"
	"sync/atomic"
	"time"
)

// ErrBulkheadFull is returned when the bulkhead is at capacity.
var ErrBulkheadFull = errors.New("bulkhead is at capacity")

// BulkheadError represents an error when the bulkhead rejects a request.
type BulkheadError struct {
	// MaxConcurrent is the maximum concurrent requests allowed.
	MaxConcurrent int

	// Current is the current number of concurrent requests.
	Current int

	// WaitTime is how long the request waited before being rejected.
	WaitTime time.Duration
}

// Error implements the error interface.
func (e *BulkheadError) Error() string {
	return "bulkhead rejected request: at capacity"
}

// Bulkhead implements the bulkhead pattern for limiting concurrent executions.
type Bulkhead struct {
	config   *BulkheadConfig
	sem      chan struct{}
	active   int64
	rejected int64
	total    int64
	mu       sync.RWMutex
}

// NewBulkhead creates a new Bulkhead with the given configuration.
//
// Example:
//
//	config := &k1s0resilience.BulkheadConfig{
//	    MaxConcurrent: 10,
//	    MaxWaitTime:   time.Second,
//	}
//	bh := k1s0resilience.NewBulkhead(config)
//
//	result, err := bh.Execute(ctx, func() (string, error) {
//	    return apiClient.Call()
//	})
func NewBulkhead(config *BulkheadConfig) *Bulkhead {
	config = config.Validate()
	return &Bulkhead{
		config: config,
		sem:    make(chan struct{}, config.MaxConcurrent),
	}
}

// NewBulkheadWithLimit creates a new Bulkhead with the given concurrency limit.
// This is a convenience function that uses default wait time (no waiting).
func NewBulkheadWithLimit(maxConcurrent int) *Bulkhead {
	return NewBulkhead(&BulkheadConfig{MaxConcurrent: maxConcurrent})
}

// Execute runs the given function within the bulkhead limits.
// If the bulkhead is at capacity, it either waits (if MaxWaitTime > 0) or returns immediately with an error.
//
// Example:
//
//	result, err := bh.Execute(ctx, func() (interface{}, error) {
//	    return httpClient.Get(url)
//	})
func (b *Bulkhead) Execute(ctx context.Context, fn func() (interface{}, error)) (interface{}, error) {
	startWait := time.Now()
	atomic.AddInt64(&b.total, 1)

	// Try to acquire a slot
	if b.config.MaxWaitTime > 0 {
		select {
		case b.sem <- struct{}{}:
			// Acquired slot
		case <-time.After(b.config.MaxWaitTime):
			atomic.AddInt64(&b.rejected, 1)
			return nil, &BulkheadError{
				MaxConcurrent: b.config.MaxConcurrent,
				Current:       int(atomic.LoadInt64(&b.active)),
				WaitTime:      time.Since(startWait),
			}
		case <-ctx.Done():
			return nil, ctx.Err()
		}
	} else {
		select {
		case b.sem <- struct{}{}:
			// Acquired slot
		default:
			atomic.AddInt64(&b.rejected, 1)
			return nil, &BulkheadError{
				MaxConcurrent: b.config.MaxConcurrent,
				Current:       int(atomic.LoadInt64(&b.active)),
				WaitTime:      time.Since(startWait),
			}
		}
	}

	// Slot acquired, track active count
	atomic.AddInt64(&b.active, 1)
	defer func() {
		atomic.AddInt64(&b.active, -1)
		<-b.sem
	}()

	// Execute the function
	return fn()
}

// ExecuteTyped runs the given function within the bulkhead limits with type safety.
//
// Example:
//
//	user, err := k1s0resilience.ExecuteBulkheadTyped(bh, ctx, func() (*User, error) {
//	    return userRepo.FindByID(id)
//	})
func ExecuteBulkheadTyped[T any](b *Bulkhead, ctx context.Context, fn func() (T, error)) (T, error) {
	var zero T
	result, err := b.Execute(ctx, func() (interface{}, error) {
		return fn()
	})
	if err != nil {
		return zero, err
	}
	if result == nil {
		return zero, nil
	}
	return result.(T), nil
}

// ExecuteFunc runs the given function within the bulkhead limits (no return value version).
//
// Example:
//
//	err := bh.ExecuteFunc(ctx, func() error {
//	    return client.Ping()
//	})
func (b *Bulkhead) ExecuteFunc(ctx context.Context, fn func() error) error {
	_, err := b.Execute(ctx, func() (interface{}, error) {
		return nil, fn()
	})
	return err
}

// Active returns the current number of active executions.
func (b *Bulkhead) Active() int {
	return int(atomic.LoadInt64(&b.active))
}

// Available returns the number of available slots.
func (b *Bulkhead) Available() int {
	return b.config.MaxConcurrent - b.Active()
}

// Stats returns the bulkhead statistics.
func (b *Bulkhead) Stats() BulkheadStats {
	return BulkheadStats{
		MaxConcurrent: b.config.MaxConcurrent,
		Active:        int(atomic.LoadInt64(&b.active)),
		Rejected:      int(atomic.LoadInt64(&b.rejected)),
		Total:         int(atomic.LoadInt64(&b.total)),
	}
}

// BulkheadStats holds the statistics of a bulkhead.
type BulkheadStats struct {
	// MaxConcurrent is the maximum concurrent requests allowed.
	MaxConcurrent int

	// Active is the current number of active requests.
	Active int

	// Rejected is the total number of rejected requests.
	Rejected int

	// Total is the total number of requests.
	Total int
}

// Utilization returns the current utilization ratio (0.0 to 1.0).
func (s BulkheadStats) Utilization() float64 {
	if s.MaxConcurrent == 0 {
		return 0
	}
	return float64(s.Active) / float64(s.MaxConcurrent)
}

// RejectionRate returns the rejection rate (0.0 to 1.0).
func (s BulkheadStats) RejectionRate() float64 {
	if s.Total == 0 {
		return 0
	}
	return float64(s.Rejected) / float64(s.Total)
}

// IsBulkheadError checks if the error is a BulkheadError.
func IsBulkheadError(err error) bool {
	var bhErr *BulkheadError
	return errors.As(err, &bhErr)
}

// BulkheadGroup manages multiple bulkheads.
type BulkheadGroup struct {
	bulkheads sync.Map
	factory   func(name string) *BulkheadConfig
}

// NewBulkheadGroup creates a new BulkheadGroup.
// The factory function is used to create configurations for new bulkheads.
func NewBulkheadGroup(factory func(name string) *BulkheadConfig) *BulkheadGroup {
	return &BulkheadGroup{
		factory: factory,
	}
}

// Get returns the bulkhead for the given name.
// If the bulkhead doesn't exist, it creates a new one.
func (g *BulkheadGroup) Get(name string) *Bulkhead {
	if bh, ok := g.bulkheads.Load(name); ok {
		return bh.(*Bulkhead)
	}

	config := g.factory(name)
	newBH := NewBulkhead(config)

	actual, _ := g.bulkheads.LoadOrStore(name, newBH)
	return actual.(*Bulkhead)
}

// Execute runs the given function through the bulkhead for the given name.
func (g *BulkheadGroup) Execute(ctx context.Context, name string, fn func() (interface{}, error)) (interface{}, error) {
	return g.Get(name).Execute(ctx, fn)
}

// Stats returns the statistics of all bulkheads.
func (g *BulkheadGroup) Stats() map[string]BulkheadStats {
	stats := make(map[string]BulkheadStats)
	g.bulkheads.Range(func(key, value interface{}) bool {
		stats[key.(string)] = value.(*Bulkhead).Stats()
		return true
	})
	return stats
}
