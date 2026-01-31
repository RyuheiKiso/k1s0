package consensus

import (
	"context"
	"time"
)

// LockGuard represents a held distributed lock. It must be released
// when the protected work is complete, either by calling Close or Unlock.
type LockGuard struct {
	// Key is the lock key.
	Key string

	// OwnerID is the unique identifier of the lock holder.
	OwnerID string

	// FenceToken is a monotonically increasing token for fencing.
	FenceToken uint64

	// AcquiredAt is when the lock was acquired.
	AcquiredAt time.Time

	// ExpiresAt is when the lock expires.
	ExpiresAt time.Time

	// releaseFunc is called by Close to release the lock.
	releaseFunc func() error
}

// Close releases the lock. It is safe to call multiple times.
// This method enables use with defer for automatic release.
func (g *LockGuard) Close() error {
	if g.releaseFunc != nil {
		err := g.releaseFunc()
		g.releaseFunc = nil
		return err
	}
	return nil
}

// IsExpired returns true if the lock has expired.
func (g *LockGuard) IsExpired() bool {
	return time.Now().After(g.ExpiresAt)
}

// DistributedLock defines the interface for distributed locking.
type DistributedLock interface {
	// TryLock attempts to acquire a lock with the given key and TTL.
	// Returns immediately with a LockGuard if successful, or an error
	// if the lock is already held.
	TryLock(ctx context.Context, key string, ttl time.Duration) (*LockGuard, error)

	// Lock attempts to acquire a lock, retrying until the timeout expires.
	// Returns a LockGuard if successful, or ErrLockTimeout if the timeout
	// is reached.
	Lock(ctx context.Context, key string, ttl time.Duration, timeout time.Duration) (*LockGuard, error)

	// Extend extends the TTL of a held lock.
	// Returns true if the extension was successful.
	Extend(ctx context.Context, guard *LockGuard, ttl time.Duration) (bool, error)

	// Unlock releases a held lock.
	Unlock(ctx context.Context, guard *LockGuard) error
}
