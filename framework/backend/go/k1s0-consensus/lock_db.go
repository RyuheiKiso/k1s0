package consensus

import (
	"context"
	"fmt"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgxpool"
)

// DbDistributedLock implements DistributedLock using PostgreSQL.
//
// It uses a table with INSERT ON CONFLICT for atomic lock acquisition
// and auto-incrementing fence tokens.
//
// Required table schema (created automatically if not present):
//
//	CREATE TABLE IF NOT EXISTS k1s0_distributed_locks (
//	    lock_key    TEXT PRIMARY KEY,
//	    owner_id    TEXT NOT NULL,
//	    fence_token BIGINT NOT NULL DEFAULT 1,
//	    acquired_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
//	    expires_at  TIMESTAMPTZ NOT NULL
//	);
type DbDistributedLock struct {
	pool   *pgxpool.Pool
	config LockConfig
}

// NewDbDistributedLock creates a new PostgreSQL-based distributed lock.
func NewDbDistributedLock(pool *pgxpool.Pool, config LockConfig) *DbDistributedLock {
	config.Validate()
	return &DbDistributedLock{
		pool:   pool,
		config: config,
	}
}

// EnsureTable creates the distributed locks table if it does not exist.
func (l *DbDistributedLock) EnsureTable(ctx context.Context) error {
	sql := fmt.Sprintf(`CREATE TABLE IF NOT EXISTS %s (
		lock_key    TEXT PRIMARY KEY,
		owner_id    TEXT NOT NULL,
		fence_token BIGINT NOT NULL DEFAULT 1,
		acquired_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
		expires_at  TIMESTAMPTZ NOT NULL
	)`, l.config.TableName)
	_, err := l.pool.Exec(ctx, sql)
	if err != nil {
		return fmt.Errorf("consensus: failed to create locks table: %w", err)
	}
	return nil
}

// TryLock attempts to acquire a lock with the given key and TTL.
func (l *DbDistributedLock) TryLock(ctx context.Context, key string, ttl time.Duration) (*LockGuard, error) {
	ownerID := uuid.New().String()
	now := time.Now()
	expiresAt := now.Add(ttl)

	sql := fmt.Sprintf(`
		INSERT INTO %s (lock_key, owner_id, fence_token, acquired_at, expires_at)
		VALUES ($1, $2, 1, $3, $4)
		ON CONFLICT (lock_key) DO UPDATE
		SET owner_id    = EXCLUDED.owner_id,
		    fence_token = %s.fence_token + 1,
		    acquired_at = EXCLUDED.acquired_at,
		    expires_at  = EXCLUDED.expires_at
		WHERE %s.expires_at < $3
		RETURNING fence_token, acquired_at, expires_at
	`, l.config.TableName, l.config.TableName, l.config.TableName)

	var guard LockGuard
	guard.Key = key
	guard.OwnerID = ownerID

	err := l.pool.QueryRow(ctx, sql, key, ownerID, now, expiresAt).
		Scan(&guard.FenceToken, &guard.AcquiredAt, &guard.ExpiresAt)
	if err != nil {
		if err == pgx.ErrNoRows {
			return nil, fmt.Errorf("consensus: lock already held: %w", ErrLockNotHeld)
		}
		return nil, fmt.Errorf("consensus: failed to acquire lock: %w", err)
	}

	guard.releaseFunc = func() error {
		return l.Unlock(context.Background(), &guard)
	}

	metricsLockAcquisitions.WithLabelValues(key, "db", "acquired").Inc()
	return &guard, nil
}

// Lock attempts to acquire a lock, retrying until the timeout expires.
func (l *DbDistributedLock) Lock(ctx context.Context, key string, ttl time.Duration, timeout time.Duration) (*LockGuard, error) {
	deadline := time.Now().Add(timeout)

	for {
		guard, err := l.TryLock(ctx, key, ttl)
		if err == nil {
			return guard, nil
		}

		if time.Now().After(deadline) {
			metricsLockAcquisitions.WithLabelValues(key, "db", "timeout").Inc()
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

// Extend extends the TTL of a held lock.
func (l *DbDistributedLock) Extend(ctx context.Context, guard *LockGuard, ttl time.Duration) (bool, error) {
	if guard == nil {
		return false, fmt.Errorf("consensus: nil guard: %w", ErrLockNotHeld)
	}

	newExpiry := time.Now().Add(ttl)

	sql := fmt.Sprintf(`
		UPDATE %s
		SET expires_at = $1
		WHERE lock_key = $2
		  AND owner_id = $3
		  AND fence_token = $4
		  AND expires_at > NOW()
	`, l.config.TableName)

	tag, err := l.pool.Exec(ctx, sql, newExpiry, guard.Key, guard.OwnerID, guard.FenceToken)
	if err != nil {
		return false, fmt.Errorf("consensus: failed to extend lock: %w", err)
	}

	if tag.RowsAffected() == 0 {
		return false, nil
	}

	guard.ExpiresAt = newExpiry
	return true, nil
}

// Unlock releases a held lock.
func (l *DbDistributedLock) Unlock(ctx context.Context, guard *LockGuard) error {
	if guard == nil {
		return fmt.Errorf("consensus: nil guard: %w", ErrLockNotHeld)
	}

	sql := fmt.Sprintf(`
		DELETE FROM %s
		WHERE lock_key = $1
		  AND owner_id = $2
		  AND fence_token = $3
	`, l.config.TableName)

	tag, err := l.pool.Exec(ctx, sql, guard.Key, guard.OwnerID, guard.FenceToken)
	if err != nil {
		return fmt.Errorf("consensus: failed to unlock: %w", err)
	}

	if tag.RowsAffected() == 0 {
		return fmt.Errorf("consensus: lock not held or expired: %w", ErrLockNotHeld)
	}

	metricsLockAcquisitions.WithLabelValues(guard.Key, "db", "released").Inc()
	return nil
}
