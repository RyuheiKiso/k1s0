package k1s0cache

import (
	"context"
	"errors"
	"sync"
	"sync/atomic"
	"time"
)

// WriteThroughConfig configures the Write-Through pattern.
type WriteThroughConfig struct {
	// DefaultTTL is the default TTL for cache entries.
	DefaultTTL time.Duration

	// FailOnCacheError determines whether to return an error if cache write fails.
	// If false (default), cache write failures are logged but don't fail the operation.
	FailOnCacheError bool

	// InvalidateOnDBError determines whether to invalidate cache if DB write fails.
	InvalidateOnDBError bool
}

// DefaultWriteThroughConfig returns the default Write-Through configuration.
func DefaultWriteThroughConfig() WriteThroughConfig {
	return WriteThroughConfig{
		DefaultTTL:          time.Hour,
		FailOnCacheError:    false,
		InvalidateOnDBError: true,
	}
}

// WriteThrough implements the Write-Through caching pattern.
// It writes to both the database and cache simultaneously.
//
// Pattern flow:
//  1. Write to database
//  2. If successful, write to cache
//
// Example:
//
//	wt := k1s0cache.NewWriteThrough(client, config)
//
//	err := wt.Write(ctx, "user:123", user, func() error {
//	    return repo.SaveUser(ctx, user)
//	})
type WriteThrough struct {
	cache  *CacheClient
	config WriteThroughConfig
}

// NewWriteThrough creates a new Write-Through cache wrapper.
func NewWriteThrough(cache *CacheClient, config WriteThroughConfig) *WriteThrough {
	return &WriteThrough{
		cache:  cache,
		config: config,
	}
}

// Write writes a value to both the database and cache.
//
// The dbWriter function is called first. If it succeeds, the value is also
// written to the cache. If the DB write fails, the cache is optionally invalidated.
func (wt *WriteThrough) Write(ctx context.Context, key string, value any, dbWriter func() error) error {
	return wt.WriteWithTTL(ctx, key, value, wt.config.DefaultTTL, dbWriter)
}

// WriteWithTTL writes a value with a specific TTL.
func (wt *WriteThrough) WriteWithTTL(ctx context.Context, key string, value any, ttl time.Duration, dbWriter func() error) error {
	// 1. Write to database first
	if err := dbWriter(); err != nil {
		// Optionally invalidate cache on DB error
		if wt.config.InvalidateOnDBError {
			_ = wt.cache.Delete(ctx, key)
		}
		return err
	}

	// 2. Write to cache
	if err := wt.cache.Set(ctx, key, value, ttl); err != nil {
		if wt.config.FailOnCacheError {
			return err
		}
		// Log warning but don't fail (in production, use proper logging)
	}

	return nil
}

// Delete removes a value from both the database and cache.
func (wt *WriteThrough) Delete(ctx context.Context, key string, dbDeleter func() error) error {
	// 1. Delete from database
	if err := dbDeleter(); err != nil {
		return err
	}

	// 2. Delete from cache
	if err := wt.cache.Delete(ctx, key); err != nil {
		if wt.config.FailOnCacheError {
			return err
		}
	}

	return nil
}

// Read retrieves a value, checking cache first then falling back to DB.
// This is identical to Cache-Aside read behavior.
func (wt *WriteThrough) Read(ctx context.Context, key string, value any, dbLoader func() error) error {
	// Try cache first
	err := wt.cache.Get(ctx, key, value)
	if err == nil {
		return nil // Cache hit
	}
	if !errors.Is(err, ErrCacheMiss) {
		// Cache error, fall back to DB
	}

	// Load from DB
	if err := dbLoader(); err != nil {
		return err
	}

	// Store in cache
	_ = wt.cache.Set(ctx, key, value, wt.config.DefaultTTL)

	return nil
}

// WriteBehindConfig configures the Write-Behind pattern.
type WriteBehindConfig struct {
	// DefaultTTL is the default TTL for cache entries.
	DefaultTTL time.Duration

	// BatchSize is the number of writes to batch before flushing.
	BatchSize int

	// FlushInterval is how often to flush pending writes.
	FlushInterval time.Duration

	// MaxRetries is the maximum number of retry attempts for failed writes.
	MaxRetries int

	// RetryDelay is the delay between retry attempts.
	RetryDelay time.Duration

	// MaxQueueSize is the maximum number of pending writes.
	MaxQueueSize int

	// FailOnCacheError determines whether to return an error if cache write fails.
	FailOnCacheError bool
}

// DefaultWriteBehindConfig returns the default Write-Behind configuration.
func DefaultWriteBehindConfig() WriteBehindConfig {
	return WriteBehindConfig{
		DefaultTTL:       time.Hour,
		BatchSize:        100,
		FlushInterval:    time.Second,
		MaxRetries:       3,
		RetryDelay:       100 * time.Millisecond,
		MaxQueueSize:     10000,
		FailOnCacheError: true,
	}
}

// WriteEntry represents a pending write operation.
type writeEntry struct {
	key     string
	writer  func() error
	retries int
}

// WriteBehindStats contains statistics for Write-Behind operations.
type WriteBehindStats struct {
	WritesSucceeded uint64
	WritesFailed    uint64
	WritesRetried   uint64
	QueueLength     int64
}

// WriteBehind implements the Write-Behind (Write-Back) caching pattern.
// It writes to the cache immediately and asynchronously writes to the database.
//
// Pattern flow:
//  1. Write to cache immediately (fast response)
//  2. Queue database write for async processing
//  3. Background worker flushes writes to database
//
// WARNING: This pattern may result in data loss if the system crashes before
// pending writes are flushed to the database.
//
// Example:
//
//	wb := k1s0cache.NewWriteBehind(client, config)
//	defer wb.Close()
//
//	err := wb.Write(ctx, "user:123", user, func() error {
//	    return repo.SaveUser(ctx, user)
//	})
type WriteBehind struct {
	cache  *CacheClient
	config WriteBehindConfig

	writeQueue chan writeEntry
	done       chan struct{}
	wg         sync.WaitGroup

	// Stats
	writesSucceeded atomic.Uint64
	writesFailed    atomic.Uint64
	writesRetried   atomic.Uint64
	queueLength     atomic.Int64
}

// NewWriteBehind creates a new Write-Behind cache wrapper.
// It starts a background worker to process pending writes.
// Call Close() when done to ensure all pending writes are flushed.
func NewWriteBehind(cache *CacheClient, config WriteBehindConfig) *WriteBehind {
	wb := &WriteBehind{
		cache:      cache,
		config:     config,
		writeQueue: make(chan writeEntry, config.MaxQueueSize),
		done:       make(chan struct{}),
	}

	wb.wg.Add(1)
	go wb.worker()

	return wb
}

// worker processes pending writes in the background.
func (wb *WriteBehind) worker() {
	defer wb.wg.Done()

	ticker := time.NewTicker(wb.config.FlushInterval)
	defer ticker.Stop()

	batch := make([]writeEntry, 0, wb.config.BatchSize)

	for {
		select {
		case entry := <-wb.writeQueue:
			wb.queueLength.Add(-1)
			batch = append(batch, entry)

			if len(batch) >= wb.config.BatchSize {
				wb.processBatch(batch)
				batch = batch[:0]
			}

		case <-ticker.C:
			if len(batch) > 0 {
				wb.processBatch(batch)
				batch = batch[:0]
			}

		case <-wb.done:
			// Process remaining items in queue
			for {
				select {
				case entry := <-wb.writeQueue:
					wb.queueLength.Add(-1)
					batch = append(batch, entry)
				default:
					if len(batch) > 0 {
						wb.processBatch(batch)
					}
					return
				}
			}
		}
	}
}

// processBatch processes a batch of pending writes.
func (wb *WriteBehind) processBatch(batch []writeEntry) {
	for _, entry := range batch {
		if err := entry.writer(); err != nil {
			if entry.retries < wb.config.MaxRetries {
				// Re-queue for retry
				wb.writesRetried.Add(1)
				entry.retries++

				// Add back to queue with delay
				go func(e writeEntry) {
					time.Sleep(wb.config.RetryDelay)
					select {
					case wb.writeQueue <- e:
						wb.queueLength.Add(1)
					default:
						// Queue full, mark as failed
						wb.writesFailed.Add(1)
					}
				}(entry)
			} else {
				wb.writesFailed.Add(1)
			}
		} else {
			wb.writesSucceeded.Add(1)
		}
	}
}

// Write writes a value to the cache immediately and queues a DB write.
func (wb *WriteBehind) Write(ctx context.Context, key string, value any, dbWriter func() error) error {
	return wb.WriteWithTTL(ctx, key, value, wb.config.DefaultTTL, dbWriter)
}

// WriteWithTTL writes a value with a specific TTL.
func (wb *WriteBehind) WriteWithTTL(ctx context.Context, key string, value any, ttl time.Duration, dbWriter func() error) error {
	// 1. Write to cache immediately
	if err := wb.cache.Set(ctx, key, value, ttl); err != nil {
		if wb.config.FailOnCacheError {
			return err
		}
	}

	// 2. Queue DB write
	entry := writeEntry{
		key:    key,
		writer: dbWriter,
	}

	select {
	case wb.writeQueue <- entry:
		wb.queueLength.Add(1)
	default:
		return errors.New("write-behind queue full")
	}

	return nil
}

// Read retrieves a value, checking cache first then falling back to DB.
func (wb *WriteBehind) Read(ctx context.Context, key string, value any, dbLoader func() error) error {
	// Try cache first
	err := wb.cache.Get(ctx, key, value)
	if err == nil {
		return nil
	}
	if !errors.Is(err, ErrCacheMiss) {
		// Cache error, fall back to DB
	}

	// Load from DB
	if err := dbLoader(); err != nil {
		return err
	}

	// Store in cache
	_ = wb.cache.Set(ctx, key, value, wb.config.DefaultTTL)

	return nil
}

// Stats returns current statistics.
func (wb *WriteBehind) Stats() WriteBehindStats {
	return WriteBehindStats{
		WritesSucceeded: wb.writesSucceeded.Load(),
		WritesFailed:    wb.writesFailed.Load(),
		WritesRetried:   wb.writesRetried.Load(),
		QueueLength:     wb.queueLength.Load(),
	}
}

// QueueLength returns the current number of pending writes.
func (wb *WriteBehind) QueueLength() int64 {
	return wb.queueLength.Load()
}

// Flush waits for all pending writes to complete.
func (wb *WriteBehind) Flush() {
	// Wait for queue to drain
	for wb.queueLength.Load() > 0 {
		time.Sleep(10 * time.Millisecond)
	}
	// Wait a bit more for processing to complete
	time.Sleep(wb.config.FlushInterval * 2)
}

// Close stops the background worker and waits for pending writes to complete.
func (wb *WriteBehind) Close() error {
	close(wb.done)
	wb.wg.Wait()
	return nil
}

// CachePattern defines the interface for cache patterns.
type CachePattern interface {
	// Write writes a value using the pattern's strategy.
	Write(ctx context.Context, key string, value any, dbWriter func() error) error

	// Read reads a value using the pattern's strategy.
	Read(ctx context.Context, key string, value any, dbLoader func() error) error
}

// Ensure patterns implement the interface
var (
	_ CachePattern = (*WriteThrough)(nil)
	_ CachePattern = (*WriteBehind)(nil)
)
