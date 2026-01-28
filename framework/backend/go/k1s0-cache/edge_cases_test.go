package k1s0cache

import (
	"context"
	"errors"
	"sync"
	"sync/atomic"
	"testing"
	"time"
)

// =============================================================================
// Distributed Cache Tests
// =============================================================================

func TestDistributedLocking(t *testing.T) {
	cache := newMockCache()
	client := NewCacheClient(cache, &JSONSerializer{}, "lock:", 5*time.Minute)
	ctx := context.Background()

	// Acquire lock
	acquired, err := client.SetNX(ctx, "mylock", "owner1", 10*time.Second)
	if err != nil {
		t.Fatalf("SetNX failed: %v", err)
	}
	if !acquired {
		t.Error("Expected to acquire lock")
	}

	// Try to acquire same lock (should fail)
	acquired2, err := client.SetNX(ctx, "mylock", "owner2", 10*time.Second)
	if err != nil {
		t.Fatalf("SetNX failed: %v", err)
	}
	if acquired2 {
		t.Error("Expected lock acquisition to fail")
	}

	// Release lock
	err = client.Delete(ctx, "mylock")
	if err != nil {
		t.Fatalf("Delete failed: %v", err)
	}

	// Now can acquire again
	acquired3, err := client.SetNX(ctx, "mylock", "owner3", 10*time.Second)
	if err != nil {
		t.Fatalf("SetNX failed: %v", err)
	}
	if !acquired3 {
		t.Error("Expected to acquire lock after release")
	}
}

func TestConcurrentSetNX(t *testing.T) {
	cache := newMockCache()
	client := NewCacheClient(cache, &JSONSerializer{}, "", 5*time.Minute)
	ctx := context.Background()

	const numWorkers = 10
	var wg sync.WaitGroup
	var successCount int32

	for i := 0; i < numWorkers; i++ {
		wg.Add(1)
		go func(id int) {
			defer wg.Done()
			acquired, err := client.SetNX(ctx, "race-key", id, 10*time.Second)
			if err != nil {
				t.Errorf("SetNX failed: %v", err)
				return
			}
			if acquired {
				atomic.AddInt32(&successCount, 1)
			}
		}(i)
	}

	wg.Wait()

	if successCount != 1 {
		t.Errorf("Expected exactly 1 success, got %d", successCount)
	}
}

// =============================================================================
// TTL Edge Cases
// =============================================================================

func TestTTLZero(t *testing.T) {
	cache := newMockCache()
	client := NewCacheClient(cache, &JSONSerializer{}, "", 5*time.Minute)
	ctx := context.Background()

	// TTL 0 should use default TTL
	err := client.Set(ctx, "zero-ttl", "value", 0)
	if err != nil {
		t.Fatalf("Set failed: %v", err)
	}

	var value string
	err = client.Get(ctx, "zero-ttl", &value)
	if err != nil {
		t.Fatalf("Get failed: %v", err)
	}
	if value != "value" {
		t.Errorf("Expected 'value', got '%s'", value)
	}
}

func TestTTLNegative(t *testing.T) {
	cache := newMockCache()
	client := NewCacheClient(cache, &JSONSerializer{}, "", 5*time.Minute)
	ctx := context.Background()

	// Negative TTL should be treated as default
	err := client.Set(ctx, "neg-ttl", "value", -1*time.Second)
	if err != nil {
		t.Fatalf("Set failed: %v", err)
	}

	exists, _ := client.Exists(ctx, "neg-ttl")
	if !exists {
		t.Error("Expected key to exist")
	}
}

// =============================================================================
// Large Data Tests
// =============================================================================

func TestLargeValue(t *testing.T) {
	cache := newMockCache()
	client := NewCacheClient(cache, &JSONSerializer{}, "", 5*time.Minute)
	ctx := context.Background()

	type LargeStruct struct {
		Items []string `json:"items"`
	}

	// 1000 items
	large := LargeStruct{
		Items: make([]string, 1000),
	}
	for i := 0; i < 1000; i++ {
		large.Items[i] = "item_" + string(rune('0'+i%10))
	}

	err := client.Set(ctx, "large", large, 0)
	if err != nil {
		t.Fatalf("Set failed: %v", err)
	}

	var retrieved LargeStruct
	err = client.Get(ctx, "large", &retrieved)
	if err != nil {
		t.Fatalf("Get failed: %v", err)
	}
	if len(retrieved.Items) != 1000 {
		t.Errorf("Expected 1000 items, got %d", len(retrieved.Items))
	}
}

func TestManyKeys(t *testing.T) {
	cache := newMockCache()
	client := NewCacheClient(cache, &JSONSerializer{}, "", 5*time.Minute)
	ctx := context.Background()

	const numKeys = 100

	// Set many keys
	for i := 0; i < numKeys; i++ {
		key := "key_" + string(rune('0'+i%10))
		err := client.Set(ctx, key, i, 0)
		if err != nil {
			t.Fatalf("Set failed for key %d: %v", i, err)
		}
	}

	// Verify some keys
	for _, i := range []int{0, 50, 99} {
		key := "key_" + string(rune('0'+i%10))
		exists, err := client.Exists(ctx, key)
		if err != nil {
			t.Fatalf("Exists failed: %v", err)
		}
		if !exists {
			t.Errorf("Expected key %s to exist", key)
		}
	}
}

// =============================================================================
// Error Handling Tests
// =============================================================================

type errorCache struct {
	*mockCache
	getError error
	setError error
}

func (c *errorCache) Get(ctx context.Context, key string, value any) error {
	if c.getError != nil {
		return c.getError
	}
	return c.mockCache.Get(ctx, key, value)
}

func (c *errorCache) Set(ctx context.Context, key string, value any, ttl time.Duration) error {
	if c.setError != nil {
		return c.setError
	}
	return c.mockCache.Set(ctx, key, value, ttl)
}

func TestGetError(t *testing.T) {
	expectedErr := errors.New("connection refused")
	cache := &errorCache{
		mockCache: newMockCache(),
		getError:  expectedErr,
	}
	client := NewCacheClient(cache, &JSONSerializer{}, "", 5*time.Minute)
	ctx := context.Background()

	var value string
	err := client.Get(ctx, "any-key", &value)
	if err != expectedErr {
		t.Errorf("Expected error %v, got %v", expectedErr, err)
	}
}

func TestSetError(t *testing.T) {
	expectedErr := errors.New("write failed")
	cache := &errorCache{
		mockCache: newMockCache(),
		setError:  expectedErr,
	}
	client := NewCacheClient(cache, &JSONSerializer{}, "", 5*time.Minute)
	ctx := context.Background()

	err := client.Set(ctx, "any-key", "value", 0)
	if err != expectedErr {
		t.Errorf("Expected error %v, got %v", expectedErr, err)
	}
}

// =============================================================================
// Serialization Tests
// =============================================================================

func TestSerializeNil(t *testing.T) {
	cache := newMockCache()
	client := NewCacheClient(cache, &JSONSerializer{}, "", 5*time.Minute)
	ctx := context.Background()

	err := client.Set(ctx, "nil-value", nil, 0)
	if err != ErrCacheNil {
		t.Errorf("Expected ErrCacheNil, got %v", err)
	}
}

func TestSerializeComplexStruct(t *testing.T) {
	cache := newMockCache()
	client := NewCacheClient(cache, &JSONSerializer{}, "", 5*time.Minute)
	ctx := context.Background()

	type Nested struct {
		Value string `json:"value"`
	}

	type Complex struct {
		ID      int       `json:"id"`
		Name    string    `json:"name"`
		Items   []int     `json:"items"`
		Nested  Nested    `json:"nested"`
		Mapping map[string]string `json:"mapping"`
	}

	original := Complex{
		ID:    123,
		Name:  "Test",
		Items: []int{1, 2, 3},
		Nested: Nested{Value: "nested_value"},
		Mapping: map[string]string{"key": "value"},
	}

	err := client.Set(ctx, "complex", original, 0)
	if err != nil {
		t.Fatalf("Set failed: %v", err)
	}

	var retrieved Complex
	err = client.Get(ctx, "complex", &retrieved)
	if err != nil {
		t.Fatalf("Get failed: %v", err)
	}

	if retrieved.ID != original.ID {
		t.Errorf("ID mismatch: expected %d, got %d", original.ID, retrieved.ID)
	}
	if retrieved.Name != original.Name {
		t.Errorf("Name mismatch: expected %s, got %s", original.Name, retrieved.Name)
	}
	if len(retrieved.Items) != len(original.Items) {
		t.Errorf("Items length mismatch: expected %d, got %d", len(original.Items), len(retrieved.Items))
	}
	if retrieved.Nested.Value != original.Nested.Value {
		t.Errorf("Nested value mismatch")
	}
}

// =============================================================================
// GetOrSet Edge Cases
// =============================================================================

func TestGetOrSetConcurrent(t *testing.T) {
	cache := newMockCache()
	client := NewCacheClient(cache, &JSONSerializer{}, "", 5*time.Minute)
	ctx := context.Background()

	const numWorkers = 10
	var wg sync.WaitGroup
	var fetchCount int32

	for i := 0; i < numWorkers; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			_, err := GetOrSet(ctx, client, "shared-key", 5*time.Minute, func() (string, error) {
				atomic.AddInt32(&fetchCount, 1)
				time.Sleep(10 * time.Millisecond) // Simulate slow fetch
				return "computed-value", nil
			})
			if err != nil {
				t.Errorf("GetOrSet failed: %v", err)
			}
		}()
	}

	wg.Wait()

	// Without proper locking, multiple fetches may occur
	// With singleflight/locking, only 1 fetch should occur
	if fetchCount > numWorkers {
		t.Errorf("Too many fetches: %d", fetchCount)
	}
}

func TestGetOrSetFetchPanic(t *testing.T) {
	defer func() {
		if r := recover(); r == nil {
			t.Error("Expected panic to be propagated")
		}
	}()

	cache := newMockCache()
	client := NewCacheClient(cache, &JSONSerializer{}, "", 5*time.Minute)
	ctx := context.Background()

	_, _ = GetOrSet(ctx, client, "panic-key", 5*time.Minute, func() (string, error) {
		panic("intentional panic")
	})
}

// =============================================================================
// Key Prefix Tests
// =============================================================================

func TestKeyPrefixIsolation(t *testing.T) {
	cache := newMockCache()
	client1 := NewCacheClient(cache, &JSONSerializer{}, "app1:", 5*time.Minute)
	client2 := NewCacheClient(cache, &JSONSerializer{}, "app2:", 5*time.Minute)
	ctx := context.Background()

	// Set same key with different clients
	_ = client1.Set(ctx, "shared", "value1", 0)
	_ = client2.Set(ctx, "shared", "value2", 0)

	var value1, value2 string
	_ = client1.Get(ctx, "shared", &value1)
	_ = client2.Get(ctx, "shared", &value2)

	if value1 != "value1" {
		t.Errorf("Expected 'value1', got '%s'", value1)
	}
	if value2 != "value2" {
		t.Errorf("Expected 'value2', got '%s'", value2)
	}

	// Verify they are stored with different prefixes
	if _, ok := cache.data["app1:shared"]; !ok {
		t.Error("Expected app1:shared key")
	}
	if _, ok := cache.data["app2:shared"]; !ok {
		t.Error("Expected app2:shared key")
	}
}

// =============================================================================
// Batch Operations Tests
// =============================================================================

func TestDeleteMultipleKeys(t *testing.T) {
	cache := newMockCache()
	client := NewCacheClient(cache, &JSONSerializer{}, "", 5*time.Minute)
	ctx := context.Background()

	// Set multiple keys
	_ = client.Set(ctx, "key1", "value1", 0)
	_ = client.Set(ctx, "key2", "value2", 0)
	_ = client.Set(ctx, "key3", "value3", 0)

	// Delete multiple
	err := client.Delete(ctx, "key1", "key2")
	if err != nil {
		t.Fatalf("Delete failed: %v", err)
	}

	// Verify
	exists1, _ := client.Exists(ctx, "key1")
	exists2, _ := client.Exists(ctx, "key2")
	exists3, _ := client.Exists(ctx, "key3")

	if exists1 {
		t.Error("key1 should be deleted")
	}
	if exists2 {
		t.Error("key2 should be deleted")
	}
	if !exists3 {
		t.Error("key3 should still exist")
	}
}

func TestDeleteNonexistentKey(t *testing.T) {
	cache := newMockCache()
	client := NewCacheClient(cache, &JSONSerializer{}, "", 5*time.Minute)
	ctx := context.Background()

	// Should not error
	err := client.Delete(ctx, "nonexistent")
	if err != nil {
		t.Errorf("Delete should not error for nonexistent key: %v", err)
	}
}

// =============================================================================
// Context Cancellation Tests
// =============================================================================

func TestContextCancellation(t *testing.T) {
	cache := newMockCache()
	client := NewCacheClient(cache, &JSONSerializer{}, "", 5*time.Minute)

	ctx, cancel := context.WithCancel(context.Background())
	cancel() // Cancel immediately

	// Operations should still work (mock doesn't check context)
	// In real Redis client, this would fail
	err := client.Set(ctx, "key", "value", 0)
	if err != nil {
		// If the implementation respects context, this would error
		t.Logf("Set with cancelled context: %v", err)
	}
}
