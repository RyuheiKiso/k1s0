package k1s0cache

import (
	"context"
	"crypto/tls"
	"errors"
	"fmt"
	"time"

	"github.com/redis/go-redis/v9"
)

// RedisCache implements Cache using Redis.
type RedisCache struct {
	client     *redis.Client
	serializer Serializer
	config     *CacheConfig
}

// NewClient creates a new Redis cache client.
//
// Example:
//
//	config, _ := k1s0cache.NewCacheConfigBuilder().
//	    Host("localhost").
//	    Port(6379).
//	    KeyPrefix("myapp:").
//	    Build()
//
//	client, err := k1s0cache.NewClient(config)
//	if err != nil {
//	    log.Fatal(err)
//	}
//	defer client.Close()
func NewClient(config *CacheConfig) (*CacheClient, error) {
	if err := config.Validate(); err != nil {
		return nil, fmt.Errorf("invalid config: %w", err)
	}

	password, err := config.GetPassword()
	if err != nil {
		return nil, fmt.Errorf("failed to get password: %w", err)
	}

	opts := &redis.Options{
		Addr:         fmt.Sprintf("%s:%d", config.Host, config.Port),
		Password:     password,
		DB:           config.Database,
		PoolSize:     config.Pool.PoolSize,
		MinIdleConns: config.Pool.MinIdleConns,
		DialTimeout:  config.Pool.DialTimeout,
		ReadTimeout:  config.Pool.ReadTimeout,
		WriteTimeout: config.Pool.WriteTimeout,
		PoolTimeout:  config.Pool.PoolTimeout,
		MaxRetries:   config.Pool.MaxRetries,
	}

	if config.TLS {
		opts.TLSConfig = &tls.Config{
			InsecureSkipVerify: config.TLSSkipVerify,
		}
	}

	client := redis.NewClient(opts)

	// Test connection
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	if err := client.Ping(ctx).Err(); err != nil {
		client.Close()
		return nil, fmt.Errorf("failed to ping Redis: %w", err)
	}

	serializer := NewSerializer(config.Serializer)

	redisCache := &RedisCache{
		client:     client,
		serializer: serializer,
		config:     config,
	}

	return NewCacheClient(redisCache, serializer, config.KeyPrefix, config.DefaultTTL), nil
}

// Get retrieves a value from Redis.
func (c *RedisCache) Get(ctx context.Context, key string, value any) error {
	data, err := c.client.Get(ctx, key).Bytes()
	if err != nil {
		if errors.Is(err, redis.Nil) {
			return ErrCacheMiss
		}
		return fmt.Errorf("redis get failed: %w", err)
	}

	if err := c.serializer.Unmarshal(data, value); err != nil {
		return fmt.Errorf("unmarshal failed: %w", err)
	}

	return nil
}

// Set stores a value in Redis.
func (c *RedisCache) Set(ctx context.Context, key string, value any, ttl time.Duration) error {
	if value == nil {
		return ErrCacheNil
	}

	data, err := c.serializer.Marshal(value)
	if err != nil {
		return fmt.Errorf("marshal failed: %w", err)
	}

	var expiration time.Duration
	if ttl > 0 {
		expiration = ttl
	} else if ttl == -1 {
		expiration = 0 // No expiration
	} else {
		expiration = c.config.DefaultTTL
	}

	if err := c.client.Set(ctx, key, data, expiration).Err(); err != nil {
		return fmt.Errorf("redis set failed: %w", err)
	}

	return nil
}

// SetNX stores a value only if the key does not exist.
func (c *RedisCache) SetNX(ctx context.Context, key string, value any, ttl time.Duration) (bool, error) {
	if value == nil {
		return false, ErrCacheNil
	}

	data, err := c.serializer.Marshal(value)
	if err != nil {
		return false, fmt.Errorf("marshal failed: %w", err)
	}

	var expiration time.Duration
	if ttl > 0 {
		expiration = ttl
	} else if ttl == -1 {
		expiration = 0
	} else {
		expiration = c.config.DefaultTTL
	}

	result, err := c.client.SetNX(ctx, key, data, expiration).Result()
	if err != nil {
		return false, fmt.Errorf("redis setnx failed: %w", err)
	}

	return result, nil
}

// Delete removes keys from Redis.
func (c *RedisCache) Delete(ctx context.Context, keys ...string) error {
	if len(keys) == 0 {
		return nil
	}

	if err := c.client.Del(ctx, keys...).Err(); err != nil {
		return fmt.Errorf("redis del failed: %w", err)
	}

	return nil
}

// Exists checks if a key exists in Redis.
func (c *RedisCache) Exists(ctx context.Context, key string) (bool, error) {
	result, err := c.client.Exists(ctx, key).Result()
	if err != nil {
		return false, fmt.Errorf("redis exists failed: %w", err)
	}

	return result > 0, nil
}

// Expire sets the TTL for a key in Redis.
func (c *RedisCache) Expire(ctx context.Context, key string, ttl time.Duration) error {
	if err := c.client.Expire(ctx, key, ttl).Err(); err != nil {
		return fmt.Errorf("redis expire failed: %w", err)
	}

	return nil
}

// TTL returns the remaining TTL for a key.
func (c *RedisCache) TTL(ctx context.Context, key string) (time.Duration, error) {
	result, err := c.client.TTL(ctx, key).Result()
	if err != nil {
		return 0, fmt.Errorf("redis ttl failed: %w", err)
	}

	return result, nil
}

// Ping checks if Redis is reachable.
func (c *RedisCache) Ping(ctx context.Context) error {
	if err := c.client.Ping(ctx).Err(); err != nil {
		return fmt.Errorf("redis ping failed: %w", err)
	}
	return nil
}

// Close closes the Redis connection.
func (c *RedisCache) Close() error {
	return c.client.Close()
}

// Incr increments a key's value by 1.
func (c *RedisCache) Incr(ctx context.Context, key string) (int64, error) {
	result, err := c.client.Incr(ctx, key).Result()
	if err != nil {
		return 0, fmt.Errorf("redis incr failed: %w", err)
	}
	return result, nil
}

// IncrBy increments a key's value by the given amount.
func (c *RedisCache) IncrBy(ctx context.Context, key string, value int64) (int64, error) {
	result, err := c.client.IncrBy(ctx, key, value).Result()
	if err != nil {
		return 0, fmt.Errorf("redis incrby failed: %w", err)
	}
	return result, nil
}

// Decr decrements a key's value by 1.
func (c *RedisCache) Decr(ctx context.Context, key string) (int64, error) {
	result, err := c.client.Decr(ctx, key).Result()
	if err != nil {
		return 0, fmt.Errorf("redis decr failed: %w", err)
	}
	return result, nil
}

// HGet gets a field from a hash.
func (c *RedisCache) HGet(ctx context.Context, key, field string, value any) error {
	data, err := c.client.HGet(ctx, key, field).Bytes()
	if err != nil {
		if errors.Is(err, redis.Nil) {
			return ErrCacheMiss
		}
		return fmt.Errorf("redis hget failed: %w", err)
	}

	if err := c.serializer.Unmarshal(data, value); err != nil {
		return fmt.Errorf("unmarshal failed: %w", err)
	}

	return nil
}

// HSet sets a field in a hash.
func (c *RedisCache) HSet(ctx context.Context, key, field string, value any) error {
	data, err := c.serializer.Marshal(value)
	if err != nil {
		return fmt.Errorf("marshal failed: %w", err)
	}

	if err := c.client.HSet(ctx, key, field, data).Err(); err != nil {
		return fmt.Errorf("redis hset failed: %w", err)
	}

	return nil
}

// HDel deletes fields from a hash.
func (c *RedisCache) HDel(ctx context.Context, key string, fields ...string) error {
	if err := c.client.HDel(ctx, key, fields...).Err(); err != nil {
		return fmt.Errorf("redis hdel failed: %w", err)
	}
	return nil
}

// SAdd adds members to a set.
func (c *RedisCache) SAdd(ctx context.Context, key string, members ...any) error {
	if err := c.client.SAdd(ctx, key, members...).Err(); err != nil {
		return fmt.Errorf("redis sadd failed: %w", err)
	}
	return nil
}

// SMembers returns all members of a set.
func (c *RedisCache) SMembers(ctx context.Context, key string) ([]string, error) {
	result, err := c.client.SMembers(ctx, key).Result()
	if err != nil {
		return nil, fmt.Errorf("redis smembers failed: %w", err)
	}
	return result, nil
}

// SIsMember checks if a value is a member of a set.
func (c *RedisCache) SIsMember(ctx context.Context, key string, member any) (bool, error) {
	result, err := c.client.SIsMember(ctx, key, member).Result()
	if err != nil {
		return false, fmt.Errorf("redis sismember failed: %w", err)
	}
	return result, nil
}

// SRem removes members from a set.
func (c *RedisCache) SRem(ctx context.Context, key string, members ...any) error {
	if err := c.client.SRem(ctx, key, members...).Err(); err != nil {
		return fmt.Errorf("redis srem failed: %w", err)
	}
	return nil
}

// Scan iterates over keys matching a pattern.
func (c *RedisCache) Scan(ctx context.Context, pattern string, count int64) ([]string, error) {
	var keys []string
	var cursor uint64

	for {
		var result []string
		var err error
		result, cursor, err = c.client.Scan(ctx, cursor, pattern, count).Result()
		if err != nil {
			return nil, fmt.Errorf("redis scan failed: %w", err)
		}

		keys = append(keys, result...)

		if cursor == 0 {
			break
		}
	}

	return keys, nil
}

// DeletePattern deletes all keys matching a pattern.
func (c *RedisCache) DeletePattern(ctx context.Context, pattern string) (int64, error) {
	keys, err := c.Scan(ctx, pattern, 100)
	if err != nil {
		return 0, err
	}

	if len(keys) == 0 {
		return 0, nil
	}

	deleted, err := c.client.Del(ctx, keys...).Result()
	if err != nil {
		return 0, fmt.Errorf("redis del failed: %w", err)
	}

	return deleted, nil
}

// GetUnderlyingClient returns the underlying Redis client.
// Use with caution - this bypasses the serialization layer.
func (c *RedisCache) GetUnderlyingClient() *redis.Client {
	return c.client
}

// Stats returns cache statistics.
type Stats struct {
	// Hits is the number of cache hits.
	Hits int64

	// Misses is the number of cache misses.
	Misses int64

	// Keys is the number of keys in the database.
	Keys int64

	// UsedMemory is the memory used by Redis.
	UsedMemory int64

	// ConnectedClients is the number of connected clients.
	ConnectedClients int64
}

// Stats returns cache statistics from Redis.
func (c *RedisCache) Stats(ctx context.Context) (*Stats, error) {
	info, err := c.client.Info(ctx, "stats", "memory", "clients", "keyspace").Result()
	if err != nil {
		return nil, fmt.Errorf("redis info failed: %w", err)
	}

	// Parse info output (simplified)
	_ = info // In a real implementation, we would parse this

	dbSize, err := c.client.DBSize(ctx).Result()
	if err != nil {
		return nil, fmt.Errorf("redis dbsize failed: %w", err)
	}

	return &Stats{
		Keys: dbSize,
	}, nil
}
