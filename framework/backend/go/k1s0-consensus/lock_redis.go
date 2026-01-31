package consensus

import (
	"context"
	"fmt"
	"time"

	"github.com/google/uuid"
	"github.com/redis/go-redis/v9"
)

// Lua script for atomic unlock: only delete if the value matches the owner.
const luaUnlock = `
if redis.call("get", KEYS[1]) == ARGV[1] then
    return redis.call("del", KEYS[1])
else
    return 0
end
`

// Lua script for atomic extend: only extend if the value matches the owner.
const luaExtend = `
if redis.call("get", KEYS[1]) == ARGV[1] then
    return redis.call("pexpire", KEYS[1], ARGV[2])
else
    return 0
end
`

// RedisDistributedLock implements DistributedLock using Redis.
//
// It uses SET NX PX for atomic lock acquisition and Lua scripts
// for safe unlock and extend operations.
type RedisDistributedLock struct {
	client redis.Cmdable
	config LockConfig
}

// NewRedisDistributedLock creates a new Redis-based distributed lock.
func NewRedisDistributedLock(client redis.Cmdable, config LockConfig) *RedisDistributedLock {
	config.Validate()
	return &RedisDistributedLock{
		client: client,
		config: config,
	}
}

// TryLock attempts to acquire a lock with the given key and TTL.
func (l *RedisDistributedLock) TryLock(ctx context.Context, key string, ttl time.Duration) (*LockGuard, error) {
	ownerID := uuid.New().String()
	redisKey := l.config.KeyPrefix + key

	ok, err := l.client.SetNX(ctx, redisKey, ownerID, ttl).Result()
	if err != nil {
		return nil, fmt.Errorf("consensus: redis SET NX failed: %w", err)
	}
	if !ok {
		return nil, fmt.Errorf("consensus: lock already held: %w", ErrLockNotHeld)
	}

	now := time.Now()
	guard := &LockGuard{
		Key:        key,
		OwnerID:    ownerID,
		AcquiredAt: now,
		ExpiresAt:  now.Add(ttl),
	}
	guard.releaseFunc = func() error {
		return l.Unlock(context.Background(), guard)
	}

	metricsLockAcquisitions.WithLabelValues(key, "redis", "acquired").Inc()
	return guard, nil
}

// Lock attempts to acquire a lock, retrying until the timeout expires.
func (l *RedisDistributedLock) Lock(ctx context.Context, key string, ttl time.Duration, timeout time.Duration) (*LockGuard, error) {
	deadline := time.Now().Add(timeout)

	for {
		guard, err := l.TryLock(ctx, key, ttl)
		if err == nil {
			return guard, nil
		}

		if time.Now().After(deadline) {
			metricsLockAcquisitions.WithLabelValues(key, "redis", "timeout").Inc()
			return nil, fmt.Errorf("consensus: timed out acquiring lock %q: %w", key, ErrLockTimeout)
		}

		select {
		case <-ctx.Done():
			return nil, ctx.Err()
		case <-time.After(l.config.RetryInterval):
			// Retry.
		}
	}
}

// Extend extends the TTL of a held lock using a Lua script for atomicity.
func (l *RedisDistributedLock) Extend(ctx context.Context, guard *LockGuard, ttl time.Duration) (bool, error) {
	if guard == nil {
		return false, fmt.Errorf("consensus: nil guard: %w", ErrLockNotHeld)
	}

	redisKey := l.config.KeyPrefix + guard.Key
	result, err := l.client.Eval(ctx, luaExtend, []string{redisKey}, guard.OwnerID, ttl.Milliseconds()).Int64()
	if err != nil {
		return false, fmt.Errorf("consensus: redis extend failed: %w", err)
	}

	if result == 0 {
		return false, nil
	}

	guard.ExpiresAt = time.Now().Add(ttl)
	return true, nil
}

// Unlock releases a held lock using a Lua script for atomicity.
func (l *RedisDistributedLock) Unlock(ctx context.Context, guard *LockGuard) error {
	if guard == nil {
		return fmt.Errorf("consensus: nil guard: %w", ErrLockNotHeld)
	}

	redisKey := l.config.KeyPrefix + guard.Key
	result, err := l.client.Eval(ctx, luaUnlock, []string{redisKey}, guard.OwnerID).Int64()
	if err != nil {
		return fmt.Errorf("consensus: redis unlock failed: %w", err)
	}

	if result == 0 {
		return fmt.Errorf("consensus: lock not held or expired: %w", ErrLockNotHeld)
	}

	metricsLockAcquisitions.WithLabelValues(guard.Key, "redis", "released").Inc()
	return nil
}
