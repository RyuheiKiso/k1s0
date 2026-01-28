package k1s0cache

import (
	"context"
	"errors"
	"time"
)

// ErrCacheMiss is returned when a key is not found in the cache.
var ErrCacheMiss = errors.New("cache miss")

// ErrCacheNil is returned when attempting to cache a nil value.
var ErrCacheNil = errors.New("cannot cache nil value")

// Cache defines the interface for cache operations.
type Cache interface {
	// Get retrieves a value from the cache.
	// Returns ErrCacheMiss if the key is not found.
	Get(ctx context.Context, key string, value any) error

	// Set stores a value in the cache with the given TTL.
	// If ttl is 0, the default TTL is used.
	// If ttl is -1, the key does not expire.
	Set(ctx context.Context, key string, value any, ttl time.Duration) error

	// SetNX stores a value only if the key does not exist.
	// Returns true if the value was set, false if the key already exists.
	SetNX(ctx context.Context, key string, value any, ttl time.Duration) (bool, error)

	// Delete removes a key from the cache.
	Delete(ctx context.Context, keys ...string) error

	// Exists checks if a key exists in the cache.
	Exists(ctx context.Context, key string) (bool, error)

	// Expire sets the TTL for an existing key.
	Expire(ctx context.Context, key string, ttl time.Duration) error

	// TTL returns the remaining TTL for a key.
	// Returns -1 if the key has no expiration.
	// Returns -2 if the key does not exist.
	TTL(ctx context.Context, key string) (time.Duration, error)

	// Ping checks if the cache is reachable.
	Ping(ctx context.Context) error

	// Close closes the cache connection.
	Close() error
}

// CacheClient is the main cache client.
type CacheClient struct {
	cache      Cache
	serializer Serializer
	keyPrefix  string
	defaultTTL time.Duration
}

// NewCacheClient creates a new CacheClient with the given cache implementation.
func NewCacheClient(cache Cache, serializer Serializer, keyPrefix string, defaultTTL time.Duration) *CacheClient {
	return &CacheClient{
		cache:      cache,
		serializer: serializer,
		keyPrefix:  keyPrefix,
		defaultTTL: defaultTTL,
	}
}

// prefixKey adds the key prefix.
func (c *CacheClient) prefixKey(key string) string {
	if c.keyPrefix == "" {
		return key
	}
	return c.keyPrefix + key
}

// resolveTTL resolves the TTL to use.
func (c *CacheClient) resolveTTL(ttl time.Duration) time.Duration {
	if ttl == 0 {
		return c.defaultTTL
	}
	return ttl
}

// Get retrieves a value from the cache.
func (c *CacheClient) Get(ctx context.Context, key string, value any) error {
	return c.cache.Get(ctx, c.prefixKey(key), value)
}

// Set stores a value in the cache.
func (c *CacheClient) Set(ctx context.Context, key string, value any, ttl time.Duration) error {
	return c.cache.Set(ctx, c.prefixKey(key), value, c.resolveTTL(ttl))
}

// SetNX stores a value only if the key does not exist.
func (c *CacheClient) SetNX(ctx context.Context, key string, value any, ttl time.Duration) (bool, error) {
	return c.cache.SetNX(ctx, c.prefixKey(key), value, c.resolveTTL(ttl))
}

// Delete removes keys from the cache.
func (c *CacheClient) Delete(ctx context.Context, keys ...string) error {
	prefixedKeys := make([]string, len(keys))
	for i, key := range keys {
		prefixedKeys[i] = c.prefixKey(key)
	}
	return c.cache.Delete(ctx, prefixedKeys...)
}

// Exists checks if a key exists.
func (c *CacheClient) Exists(ctx context.Context, key string) (bool, error) {
	return c.cache.Exists(ctx, c.prefixKey(key))
}

// Expire sets the TTL for a key.
func (c *CacheClient) Expire(ctx context.Context, key string, ttl time.Duration) error {
	return c.cache.Expire(ctx, c.prefixKey(key), ttl)
}

// TTL returns the remaining TTL for a key.
func (c *CacheClient) TTL(ctx context.Context, key string) (time.Duration, error) {
	return c.cache.TTL(ctx, c.prefixKey(key))
}

// Ping checks if the cache is reachable.
func (c *CacheClient) Ping(ctx context.Context) error {
	return c.cache.Ping(ctx)
}

// Close closes the cache connection.
func (c *CacheClient) Close() error {
	return c.cache.Close()
}

// GetOrSet retrieves a value from the cache, or sets it using the provided function.
// This implements the cache-aside pattern.
//
// Example:
//
//	user, err := k1s0cache.GetOrSet(ctx, client, "user:123", 5*time.Minute, func() (*User, error) {
//	    return repo.FindByID(ctx, "123")
//	})
func GetOrSet[T any](ctx context.Context, client *CacheClient, key string, ttl time.Duration, fn func() (T, error)) (T, error) {
	var value T

	// Try to get from cache
	err := client.Get(ctx, key, &value)
	if err == nil {
		return value, nil
	}
	if !errors.Is(err, ErrCacheMiss) {
		// Log error but continue to fetch from source
		// Could add observability here
	}

	// Fetch from source
	value, err = fn()
	if err != nil {
		return value, err
	}

	// Store in cache (fire and forget)
	_ = client.Set(ctx, key, value, ttl)

	return value, nil
}

// GetOrSetWithLock retrieves a value from the cache, or sets it using the provided function.
// Uses a distributed lock to prevent thundering herd.
func GetOrSetWithLock[T any](ctx context.Context, client *CacheClient, key string, ttl time.Duration, lockTTL time.Duration, fn func() (T, error)) (T, error) {
	var value T

	// Try to get from cache
	err := client.Get(ctx, key, &value)
	if err == nil {
		return value, nil
	}
	if !errors.Is(err, ErrCacheMiss) {
		// Log error but continue
	}

	// Try to acquire lock
	lockKey := key + ":lock"
	locked, err := client.SetNX(ctx, lockKey, "1", lockTTL)
	if err != nil {
		// Lock failed, try to get from cache again or fetch directly
		err := client.Get(ctx, key, &value)
		if err == nil {
			return value, nil
		}
		// Fall through to fetch from source
	}

	if !locked {
		// Another process is fetching, wait and retry
		time.Sleep(50 * time.Millisecond)
		err := client.Get(ctx, key, &value)
		if err == nil {
			return value, nil
		}
		// Fall through to fetch from source
	}

	defer func() {
		if locked {
			_ = client.Delete(ctx, lockKey)
		}
	}()

	// Fetch from source
	value, err = fn()
	if err != nil {
		return value, err
	}

	// Store in cache
	_ = client.Set(ctx, key, value, ttl)

	return value, nil
}

// Invalidate deletes the given keys from the cache.
func Invalidate(ctx context.Context, client *CacheClient, keys ...string) error {
	return client.Delete(ctx, keys...)
}

// InvalidatePattern deletes all keys matching the pattern.
// Note: This operation may be slow for large datasets.
func InvalidatePattern(ctx context.Context, client *CacheClient, pattern string) error {
	// This requires the underlying cache to support pattern deletion
	// For Redis, we would use SCAN + DEL
	// For now, this is a no-op placeholder
	return errors.New("pattern invalidation not implemented")
}
