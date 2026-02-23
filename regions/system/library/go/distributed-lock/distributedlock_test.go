package distributedlock_test

import (
	"context"
	"testing"
	"time"

	"github.com/k1s0-platform/system-library-go-distributed-lock"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestAcquire_Success(t *testing.T) {
	l := distributedlock.NewInMemoryLock()
	ctx := context.Background()
	guard, err := l.Acquire(ctx, "key1", time.Second)
	require.NoError(t, err)
	assert.Equal(t, "key1", guard.Key)
	assert.NotEmpty(t, guard.Token)
}

func TestAcquire_Duplicate(t *testing.T) {
	l := distributedlock.NewInMemoryLock()
	ctx := context.Background()
	_, err := l.Acquire(ctx, "key1", time.Second)
	require.NoError(t, err)

	_, err = l.Acquire(ctx, "key1", time.Second)
	assert.ErrorIs(t, err, distributedlock.ErrAlreadyLocked)
}

func TestAcquire_AfterExpiry(t *testing.T) {
	l := distributedlock.NewInMemoryLock()
	ctx := context.Background()
	_, err := l.Acquire(ctx, "key1", 50*time.Millisecond)
	require.NoError(t, err)

	time.Sleep(60 * time.Millisecond)

	guard, err := l.Acquire(ctx, "key1", time.Second)
	require.NoError(t, err)
	assert.NotEmpty(t, guard.Token)
}

func TestRelease_Success(t *testing.T) {
	l := distributedlock.NewInMemoryLock()
	ctx := context.Background()
	guard, _ := l.Acquire(ctx, "key1", time.Second)

	err := l.Release(ctx, guard)
	assert.NoError(t, err)

	locked, _ := l.IsLocked(ctx, "key1")
	assert.False(t, locked)
}

func TestRelease_TokenMismatch(t *testing.T) {
	l := distributedlock.NewInMemoryLock()
	ctx := context.Background()
	_, _ = l.Acquire(ctx, "key1", time.Second)

	wrongGuard := &distributedlock.LockGuard{Key: "key1", Token: "wrong"}
	err := l.Release(ctx, wrongGuard)
	assert.ErrorIs(t, err, distributedlock.ErrTokenMismatch)
}

func TestRelease_NotFound(t *testing.T) {
	l := distributedlock.NewInMemoryLock()
	ctx := context.Background()
	guard := &distributedlock.LockGuard{Key: "missing", Token: "abc"}
	err := l.Release(ctx, guard)
	assert.ErrorIs(t, err, distributedlock.ErrLockNotFound)
}

func TestIsLocked(t *testing.T) {
	l := distributedlock.NewInMemoryLock()
	ctx := context.Background()

	locked, _ := l.IsLocked(ctx, "key1")
	assert.False(t, locked)

	_, _ = l.Acquire(ctx, "key1", time.Second)
	locked, _ = l.IsLocked(ctx, "key1")
	assert.True(t, locked)
}
