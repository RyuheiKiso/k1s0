package k1s0cache

import (
	"context"
	"errors"
	"os"
	"testing"
	"time"
)

// =============================================================================
// Config Tests
// =============================================================================

func TestCacheConfig_Validate_Success(t *testing.T) {
	config := DefaultCacheConfig()

	err := config.Validate()
	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
}

func TestCacheConfig_Validate_MissingHost(t *testing.T) {
	config := DefaultCacheConfig()
	config.Host = ""

	err := config.Validate()
	if err == nil {
		t.Error("expected error for missing host")
	}
}

func TestCacheConfig_Validate_InvalidPort(t *testing.T) {
	config := DefaultCacheConfig()
	config.Port = 0

	err := config.Validate()
	if err == nil {
		t.Error("expected error for invalid port")
	}
}

func TestCacheConfig_Validate_InvalidDatabase(t *testing.T) {
	config := DefaultCacheConfig()
	config.Database = 16

	err := config.Validate()
	if err == nil {
		t.Error("expected error for invalid database")
	}
}

func TestCacheConfig_Validate_InvalidSerializer(t *testing.T) {
	config := DefaultCacheConfig()
	config.Serializer = "invalid"

	err := config.Validate()
	if err == nil {
		t.Error("expected error for invalid serializer")
	}
}

func TestCacheConfig_GetPassword_Direct(t *testing.T) {
	config := &CacheConfig{Password: "direct_secret"}

	password, err := config.GetPassword()
	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	if password != "direct_secret" {
		t.Errorf("expected 'direct_secret', got '%s'", password)
	}
}

func TestCacheConfig_GetPassword_FromFile(t *testing.T) {
	// Create temp file with password
	tmpFile, err := os.CreateTemp("", "password")
	if err != nil {
		t.Fatal(err)
	}
	defer os.Remove(tmpFile.Name())

	if _, err := tmpFile.WriteString("file_secret\n"); err != nil {
		t.Fatal(err)
	}
	tmpFile.Close()

	config := &CacheConfig{PasswordFile: tmpFile.Name()}

	password, err := config.GetPassword()
	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	if password != "file_secret" {
		t.Errorf("expected 'file_secret', got '%s'", password)
	}
}

func TestCacheConfig_GetPassword_NoPassword(t *testing.T) {
	config := &CacheConfig{}

	password, err := config.GetPassword()
	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	if password != "" {
		t.Errorf("expected empty password, got '%s'", password)
	}
}

func TestCacheConfigBuilder(t *testing.T) {
	config, err := NewCacheConfigBuilder().
		Host("myhost").
		Port(6380).
		Database(1).
		KeyPrefix("test:").
		DefaultTTL(10 * time.Minute).
		Serializer("msgpack").
		PoolSize(20).
		Build()

	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	if config.Host != "myhost" {
		t.Errorf("expected host 'myhost', got '%s'", config.Host)
	}
	if config.Port != 6380 {
		t.Errorf("expected port 6380, got %d", config.Port)
	}
	if config.Database != 1 {
		t.Errorf("expected database 1, got %d", config.Database)
	}
	if config.KeyPrefix != "test:" {
		t.Errorf("expected prefix 'test:', got '%s'", config.KeyPrefix)
	}
	if config.Serializer != "msgpack" {
		t.Errorf("expected serializer 'msgpack', got '%s'", config.Serializer)
	}
}

func TestPoolConfig_Validate(t *testing.T) {
	config := &PoolConfig{
		PoolSize:     0,
		MinIdleConns: -1,
		DialTimeout:  0,
	}

	validated := config.Validate()

	if validated.PoolSize != 10 {
		t.Errorf("expected PoolSize 10, got %d", validated.PoolSize)
	}
	if validated.MinIdleConns != 5 {
		t.Errorf("expected MinIdleConns 5, got %d", validated.MinIdleConns)
	}
	if validated.DialTimeout != 5*time.Second {
		t.Errorf("expected DialTimeout 5s, got %v", validated.DialTimeout)
	}
}

// =============================================================================
// Serializer Tests
// =============================================================================

type testStruct struct {
	Name  string `json:"name" msgpack:"name"`
	Value int    `json:"value" msgpack:"value"`
}

func TestJSONSerializer(t *testing.T) {
	s := &JSONSerializer{}

	if s.Name() != "json" {
		t.Errorf("expected name 'json', got '%s'", s.Name())
	}

	original := testStruct{Name: "test", Value: 42}
	data, err := s.Marshal(original)
	if err != nil {
		t.Fatalf("marshal failed: %v", err)
	}

	var decoded testStruct
	if err := s.Unmarshal(data, &decoded); err != nil {
		t.Fatalf("unmarshal failed: %v", err)
	}

	if decoded.Name != original.Name || decoded.Value != original.Value {
		t.Errorf("expected %+v, got %+v", original, decoded)
	}
}

func TestMsgpackSerializer(t *testing.T) {
	s := &MsgpackSerializer{}

	if s.Name() != "msgpack" {
		t.Errorf("expected name 'msgpack', got '%s'", s.Name())
	}

	original := testStruct{Name: "test", Value: 42}
	data, err := s.Marshal(original)
	if err != nil {
		t.Fatalf("marshal failed: %v", err)
	}

	var decoded testStruct
	if err := s.Unmarshal(data, &decoded); err != nil {
		t.Fatalf("unmarshal failed: %v", err)
	}

	if decoded.Name != original.Name || decoded.Value != original.Value {
		t.Errorf("expected %+v, got %+v", original, decoded)
	}
}

func TestNewSerializer(t *testing.T) {
	jsonSerializer := NewSerializer("json")
	if jsonSerializer.Name() != "json" {
		t.Error("expected JSON serializer")
	}

	msgpackSerializer := NewSerializer("msgpack")
	if msgpackSerializer.Name() != "msgpack" {
		t.Error("expected Msgpack serializer")
	}

	defaultSerializer := NewSerializer("unknown")
	if defaultSerializer.Name() != "json" {
		t.Error("expected default JSON serializer")
	}
}

// =============================================================================
// Mock Cache Tests
// =============================================================================

type mockCache struct {
	data map[string][]byte
}

func newMockCache() *mockCache {
	return &mockCache{
		data: make(map[string][]byte),
	}
}

func (c *mockCache) Get(ctx context.Context, key string, value any) error {
	data, ok := c.data[key]
	if !ok {
		return ErrCacheMiss
	}
	serializer := &JSONSerializer{}
	return serializer.Unmarshal(data, value)
}

func (c *mockCache) Set(ctx context.Context, key string, value any, ttl time.Duration) error {
	if value == nil {
		return ErrCacheNil
	}
	serializer := &JSONSerializer{}
	data, err := serializer.Marshal(value)
	if err != nil {
		return err
	}
	c.data[key] = data
	return nil
}

func (c *mockCache) SetNX(ctx context.Context, key string, value any, ttl time.Duration) (bool, error) {
	if _, ok := c.data[key]; ok {
		return false, nil
	}
	return true, c.Set(ctx, key, value, ttl)
}

func (c *mockCache) Delete(ctx context.Context, keys ...string) error {
	for _, key := range keys {
		delete(c.data, key)
	}
	return nil
}

func (c *mockCache) Exists(ctx context.Context, key string) (bool, error) {
	_, ok := c.data[key]
	return ok, nil
}

func (c *mockCache) Expire(ctx context.Context, key string, ttl time.Duration) error {
	return nil
}

func (c *mockCache) TTL(ctx context.Context, key string) (time.Duration, error) {
	if _, ok := c.data[key]; !ok {
		return -2, nil
	}
	return -1, nil
}

func (c *mockCache) Ping(ctx context.Context) error {
	return nil
}

func (c *mockCache) Close() error {
	return nil
}

func TestCacheClient_GetSet(t *testing.T) {
	cache := newMockCache()
	client := NewCacheClient(cache, &JSONSerializer{}, "test:", 5*time.Minute)
	ctx := context.Background()

	// Set a value
	original := testStruct{Name: "hello", Value: 123}
	err := client.Set(ctx, "key1", original, 0)
	if err != nil {
		t.Fatalf("set failed: %v", err)
	}

	// Get the value
	var retrieved testStruct
	err = client.Get(ctx, "key1", &retrieved)
	if err != nil {
		t.Fatalf("get failed: %v", err)
	}

	if retrieved.Name != original.Name || retrieved.Value != original.Value {
		t.Errorf("expected %+v, got %+v", original, retrieved)
	}
}

func TestCacheClient_GetMiss(t *testing.T) {
	cache := newMockCache()
	client := NewCacheClient(cache, &JSONSerializer{}, "", 5*time.Minute)
	ctx := context.Background()

	var value testStruct
	err := client.Get(ctx, "nonexistent", &value)
	if !errors.Is(err, ErrCacheMiss) {
		t.Errorf("expected ErrCacheMiss, got %v", err)
	}
}

func TestCacheClient_Delete(t *testing.T) {
	cache := newMockCache()
	client := NewCacheClient(cache, &JSONSerializer{}, "", 5*time.Minute)
	ctx := context.Background()

	// Set and delete
	_ = client.Set(ctx, "key1", "value1", 0)
	err := client.Delete(ctx, "key1")
	if err != nil {
		t.Fatalf("delete failed: %v", err)
	}

	// Verify deleted
	var value string
	err = client.Get(ctx, "key1", &value)
	if !errors.Is(err, ErrCacheMiss) {
		t.Error("expected cache miss after delete")
	}
}

func TestCacheClient_SetNX(t *testing.T) {
	cache := newMockCache()
	client := NewCacheClient(cache, &JSONSerializer{}, "", 5*time.Minute)
	ctx := context.Background()

	// First SetNX should succeed
	ok, err := client.SetNX(ctx, "key1", "value1", 0)
	if err != nil {
		t.Fatalf("setnx failed: %v", err)
	}
	if !ok {
		t.Error("expected first SetNX to succeed")
	}

	// Second SetNX should fail
	ok, err = client.SetNX(ctx, "key1", "value2", 0)
	if err != nil {
		t.Fatalf("setnx failed: %v", err)
	}
	if ok {
		t.Error("expected second SetNX to fail")
	}
}

func TestCacheClient_Exists(t *testing.T) {
	cache := newMockCache()
	client := NewCacheClient(cache, &JSONSerializer{}, "", 5*time.Minute)
	ctx := context.Background()

	// Should not exist
	exists, err := client.Exists(ctx, "key1")
	if err != nil {
		t.Fatalf("exists failed: %v", err)
	}
	if exists {
		t.Error("expected key not to exist")
	}

	// Set and check again
	_ = client.Set(ctx, "key1", "value1", 0)
	exists, err = client.Exists(ctx, "key1")
	if err != nil {
		t.Fatalf("exists failed: %v", err)
	}
	if !exists {
		t.Error("expected key to exist")
	}
}

func TestCacheClient_KeyPrefix(t *testing.T) {
	cache := newMockCache()
	client := NewCacheClient(cache, &JSONSerializer{}, "myapp:", 5*time.Minute)
	ctx := context.Background()

	_ = client.Set(ctx, "key1", "value1", 0)

	// Check that the key was stored with prefix
	if _, ok := cache.data["myapp:key1"]; !ok {
		t.Error("expected key to be stored with prefix")
	}
	if _, ok := cache.data["key1"]; ok {
		t.Error("key should not be stored without prefix")
	}
}

func TestGetOrSet(t *testing.T) {
	cache := newMockCache()
	client := NewCacheClient(cache, &JSONSerializer{}, "", 5*time.Minute)
	ctx := context.Background()

	fetchCount := 0
	fetchFn := func() (testStruct, error) {
		fetchCount++
		return testStruct{Name: "fetched", Value: 100}, nil
	}

	// First call should fetch
	result1, err := GetOrSet(ctx, client, "key1", 5*time.Minute, fetchFn)
	if err != nil {
		t.Fatalf("GetOrSet failed: %v", err)
	}
	if fetchCount != 1 {
		t.Errorf("expected 1 fetch, got %d", fetchCount)
	}
	if result1.Name != "fetched" {
		t.Errorf("expected name 'fetched', got '%s'", result1.Name)
	}

	// Second call should use cache
	result2, err := GetOrSet(ctx, client, "key1", 5*time.Minute, fetchFn)
	if err != nil {
		t.Fatalf("GetOrSet failed: %v", err)
	}
	if fetchCount != 1 {
		t.Errorf("expected still 1 fetch, got %d", fetchCount)
	}
	if result2.Name != "fetched" {
		t.Errorf("expected name 'fetched', got '%s'", result2.Name)
	}
}

func TestGetOrSet_FetchError(t *testing.T) {
	cache := newMockCache()
	client := NewCacheClient(cache, &JSONSerializer{}, "", 5*time.Minute)
	ctx := context.Background()

	fetchErr := errors.New("fetch error")
	fetchFn := func() (testStruct, error) {
		return testStruct{}, fetchErr
	}

	_, err := GetOrSet(ctx, client, "key1", 5*time.Minute, fetchFn)
	if !errors.Is(err, fetchErr) {
		t.Errorf("expected fetch error, got %v", err)
	}
}

// =============================================================================
// Pattern Delete Tests
// =============================================================================

// mockCacheWithPatternDelete implements both Cache and PatternDeleter interfaces.
type mockCacheWithPatternDelete struct {
	*mockCache
}

func newMockCacheWithPatternDelete() *mockCacheWithPatternDelete {
	return &mockCacheWithPatternDelete{
		mockCache: newMockCache(),
	}
}

func (c *mockCacheWithPatternDelete) DeletePattern(ctx context.Context, pattern string) (int64, error) {
	var deleted int64
	// Simple pattern matching: just check prefix for "*" suffix
	prefix := pattern
	if len(pattern) > 0 && pattern[len(pattern)-1] == '*' {
		prefix = pattern[:len(pattern)-1]
	}

	keysToDelete := []string{}
	for key := range c.data {
		if pattern == key || (prefix != pattern && len(key) >= len(prefix) && key[:len(prefix)] == prefix) {
			keysToDelete = append(keysToDelete, key)
		}
	}

	for _, key := range keysToDelete {
		delete(c.data, key)
		deleted++
	}

	return deleted, nil
}

func (c *mockCacheWithPatternDelete) Scan(ctx context.Context, pattern string, count int64) ([]string, error) {
	var keys []string
	prefix := pattern
	if len(pattern) > 0 && pattern[len(pattern)-1] == '*' {
		prefix = pattern[:len(pattern)-1]
	}

	for key := range c.data {
		if pattern == key || (prefix != pattern && len(key) >= len(prefix) && key[:len(prefix)] == prefix) {
			keys = append(keys, key)
		}
	}

	return keys, nil
}

func TestCacheClient_DeletePattern(t *testing.T) {
	cache := newMockCacheWithPatternDelete()
	client := NewCacheClient(cache, &JSONSerializer{}, "app:", 5*time.Minute)
	ctx := context.Background()

	// Set some values
	_ = client.Set(ctx, "user:1", "value1", 0)
	_ = client.Set(ctx, "user:2", "value2", 0)
	_ = client.Set(ctx, "user:3", "value3", 0)
	_ = client.Set(ctx, "session:abc", "sessiondata", 0)

	// Delete all user keys
	deleted, err := client.DeletePattern(ctx, "user:*")
	if err != nil {
		t.Fatalf("DeletePattern failed: %v", err)
	}
	if deleted != 3 {
		t.Errorf("expected 3 deleted, got %d", deleted)
	}

	// Verify user keys are deleted
	exists, _ := client.Exists(ctx, "user:1")
	if exists {
		t.Error("expected user:1 to be deleted")
	}

	// Verify session key is still there
	exists, _ = client.Exists(ctx, "session:abc")
	if !exists {
		t.Error("expected session:abc to still exist")
	}
}

func TestScanKeys(t *testing.T) {
	cache := newMockCacheWithPatternDelete()
	client := NewCacheClient(cache, &JSONSerializer{}, "", 5*time.Minute)
	ctx := context.Background()

	// Set some values
	_ = client.Set(ctx, "user:1", "value1", 0)
	_ = client.Set(ctx, "user:2", "value2", 0)
	_ = client.Set(ctx, "other:key", "value", 0)

	// Scan user keys
	keys, err := ScanKeys(ctx, client, "user:*")
	if err != nil {
		t.Fatalf("ScanKeys failed: %v", err)
	}
	if len(keys) != 2 {
		t.Errorf("expected 2 keys, got %d", len(keys))
	}
}

func TestInvalidatePattern_NotSupported(t *testing.T) {
	// Use mockCache which does NOT implement PatternDeleter
	cache := newMockCache()
	client := NewCacheClient(cache, &JSONSerializer{}, "", 5*time.Minute)
	ctx := context.Background()

	_, err := InvalidatePattern(ctx, client, "user:*")
	if err == nil {
		t.Error("expected error for unsupported pattern deletion")
	}
}
