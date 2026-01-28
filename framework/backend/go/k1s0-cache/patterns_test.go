package k1s0cache

import (
	"context"
	"errors"
	"sync"
	"sync/atomic"
	"testing"
	"time"
)

// MockCache is a simple in-memory cache for testing.
type MockCache struct {
	mu   sync.RWMutex
	data map[string][]byte
}

func NewMockCache() *MockCache {
	return &MockCache{
		data: make(map[string][]byte),
	}
}

func (m *MockCache) Get(ctx context.Context, key string, value any) error {
	m.mu.RLock()
	defer m.mu.RUnlock()

	data, ok := m.data[key]
	if !ok {
		return ErrCacheMiss
	}

	// Simple string handling for tests
	if ptr, ok := value.(*string); ok {
		*ptr = string(data)
		return nil
	}

	return errors.New("unsupported type")
}

func (m *MockCache) Set(ctx context.Context, key string, value any, ttl time.Duration) error {
	m.mu.Lock()
	defer m.mu.Unlock()

	// Simple string handling for tests
	if s, ok := value.(string); ok {
		m.data[key] = []byte(s)
		return nil
	}

	return errors.New("unsupported type")
}

func (m *MockCache) SetNX(ctx context.Context, key string, value any, ttl time.Duration) (bool, error) {
	m.mu.Lock()
	defer m.mu.Unlock()

	if _, exists := m.data[key]; exists {
		return false, nil
	}

	if s, ok := value.(string); ok {
		m.data[key] = []byte(s)
		return true, nil
	}

	return false, errors.New("unsupported type")
}

func (m *MockCache) Delete(ctx context.Context, keys ...string) error {
	m.mu.Lock()
	defer m.mu.Unlock()

	for _, key := range keys {
		delete(m.data, key)
	}
	return nil
}

func (m *MockCache) Exists(ctx context.Context, key string) (bool, error) {
	m.mu.RLock()
	defer m.mu.RUnlock()

	_, exists := m.data[key]
	return exists, nil
}

func (m *MockCache) Expire(ctx context.Context, key string, ttl time.Duration) error {
	return nil
}

func (m *MockCache) TTL(ctx context.Context, key string) (time.Duration, error) {
	return -1, nil
}

func (m *MockCache) Ping(ctx context.Context) error {
	return nil
}

func (m *MockCache) Close() error {
	return nil
}

// createMockCacheClient creates a CacheClient with a mock cache for testing.
func createMockCacheClient() (*CacheClient, *MockCache) {
	mock := NewMockCache()
	client := &CacheClient{
		cache:      mock,
		serializer: NewSerializer("json"),
		keyPrefix:  "test:",
		defaultTTL: time.Hour,
	}
	return client, mock
}

func TestWriteThroughBasic(t *testing.T) {
	client, mock := createMockCacheClient()
	wt := NewWriteThrough(client, DefaultWriteThroughConfig())

	var dbWriteCount int32
	ctx := context.Background()

	// Write through
	err := wt.Write(ctx, "user:1", "Alice", func() error {
		atomic.AddInt32(&dbWriteCount, 1)
		return nil
	})
	if err != nil {
		t.Fatalf("Write failed: %v", err)
	}

	// Check DB was written
	if dbWriteCount != 1 {
		t.Errorf("Expected DB write count 1, got %d", dbWriteCount)
	}

	// Check cache was written
	var value string
	if err := mock.Get(ctx, "test:user:1", &value); err != nil {
		t.Errorf("Cache should contain value: %v", err)
	}
	if value != "Alice" {
		t.Errorf("Expected 'Alice', got '%s'", value)
	}
}

func TestWriteThroughDBFailure(t *testing.T) {
	client, mock := createMockCacheClient()
	wt := NewWriteThrough(client, DefaultWriteThroughConfig())

	ctx := context.Background()
	dbError := errors.New("DB write failed")

	// Write should fail
	err := wt.Write(ctx, "user:1", "Alice", func() error {
		return dbError
	})
	if err != dbError {
		t.Fatalf("Expected DB error, got: %v", err)
	}

	// Cache should not contain value
	var value string
	if err := mock.Get(ctx, "test:user:1", &value); !errors.Is(err, ErrCacheMiss) {
		t.Errorf("Cache should be empty")
	}
}

func TestWriteBehindBasic(t *testing.T) {
	client, mock := createMockCacheClient()
	config := DefaultWriteBehindConfig()
	config.FlushInterval = 50 * time.Millisecond
	wb := NewWriteBehind(client, config)
	defer wb.Close()

	var dbWriteCount int32
	ctx := context.Background()

	// Write behind
	err := wb.Write(ctx, "user:1", "Bob", func() error {
		atomic.AddInt32(&dbWriteCount, 1)
		return nil
	})
	if err != nil {
		t.Fatalf("Write failed: %v", err)
	}

	// Cache should be written immediately
	var value string
	if err := mock.Get(ctx, "test:user:1", &value); err != nil {
		t.Errorf("Cache should contain value: %v", err)
	}
	if value != "Bob" {
		t.Errorf("Expected 'Bob', got '%s'", value)
	}

	// Wait for async DB write
	time.Sleep(200 * time.Millisecond)

	// DB should be written
	if atomic.LoadInt32(&dbWriteCount) != 1 {
		t.Errorf("Expected DB write count 1, got %d", dbWriteCount)
	}
}

func TestWriteBehindStats(t *testing.T) {
	client, _ := createMockCacheClient()
	config := DefaultWriteBehindConfig()
	config.FlushInterval = 50 * time.Millisecond
	wb := NewWriteBehind(client, config)
	defer wb.Close()

	ctx := context.Background()

	// Write multiple items
	for i := 0; i < 5; i++ {
		key := "key:" + string(rune('0'+i))
		value := "value" + string(rune('0'+i))
		_ = wb.Write(ctx, key, value, func() error {
			return nil
		})
	}

	// Wait for processing
	time.Sleep(200 * time.Millisecond)

	stats := wb.Stats()
	if stats.WritesSucceeded != 5 {
		t.Errorf("Expected 5 successful writes, got %d", stats.WritesSucceeded)
	}
	if stats.WritesFailed != 0 {
		t.Errorf("Expected 0 failed writes, got %d", stats.WritesFailed)
	}
}

func TestWriteBehindRetry(t *testing.T) {
	client, _ := createMockCacheClient()
	config := DefaultWriteBehindConfig()
	config.FlushInterval = 50 * time.Millisecond
	config.RetryDelay = 10 * time.Millisecond
	config.MaxRetries = 2
	wb := NewWriteBehind(client, config)
	defer wb.Close()

	ctx := context.Background()
	var attempts int32

	// Write with failing DB writer
	_ = wb.Write(ctx, "failing:key", "value", func() error {
		if atomic.AddInt32(&attempts, 1) < 3 {
			return errors.New("temporary failure")
		}
		return nil
	})

	// Wait for retries
	time.Sleep(500 * time.Millisecond)

	finalAttempts := atomic.LoadInt32(&attempts)
	if finalAttempts < 2 {
		t.Errorf("Expected at least 2 attempts, got %d", finalAttempts)
	}
}
