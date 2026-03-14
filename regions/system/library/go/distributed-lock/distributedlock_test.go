package distributedlock_test

import (
	"context"
	"testing"
	"time"

	"github.com/k1s0-platform/system-library-go-distributed-lock"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// ロックの取得が成功し、キーとトークンを持つLockGuardが返ることを確認する。
func TestAcquire_Success(t *testing.T) {
	l := distributedlock.NewInMemoryLock()
	ctx := context.Background()
	guard, err := l.Acquire(ctx, "key1", time.Second)
	require.NoError(t, err)
	assert.Equal(t, "key1", guard.Key)
	assert.NotEmpty(t, guard.Token)
}

// 同じキーに対して2回Acquireを呼んだ場合にErrAlreadyLockedが返ることを確認する。
func TestAcquire_Duplicate(t *testing.T) {
	l := distributedlock.NewInMemoryLock()
	ctx := context.Background()
	_, err := l.Acquire(ctx, "key1", time.Second)
	require.NoError(t, err)

	_, err = l.Acquire(ctx, "key1", time.Second)
	assert.ErrorIs(t, err, distributedlock.ErrAlreadyLocked)
}

// ロックのTTLが切れた後に同じキーで再取得できることを確認する。
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

// 正しいLockGuardでReleaseを呼んだ場合にロックが解放されることを確認する。
func TestRelease_Success(t *testing.T) {
	l := distributedlock.NewInMemoryLock()
	ctx := context.Background()
	guard, _ := l.Acquire(ctx, "key1", time.Second)

	err := l.Release(ctx, guard)
	assert.NoError(t, err)

	locked, _ := l.IsLocked(ctx, "key1")
	assert.False(t, locked)
}

// トークンが一致しないLockGuardでReleaseを呼んだ場合にErrTokenMismatchが返ることを確認する。
func TestRelease_TokenMismatch(t *testing.T) {
	l := distributedlock.NewInMemoryLock()
	ctx := context.Background()
	_, _ = l.Acquire(ctx, "key1", time.Second)

	wrongGuard := &distributedlock.LockGuard{Key: "key1", Token: "wrong"}
	err := l.Release(ctx, wrongGuard)
	assert.ErrorIs(t, err, distributedlock.ErrTokenMismatch)
}

// 存在しないキーのLockGuardでReleaseを呼んだ場合にErrLockNotFoundが返ることを確認する。
func TestRelease_NotFound(t *testing.T) {
	l := distributedlock.NewInMemoryLock()
	ctx := context.Background()
	guard := &distributedlock.LockGuard{Key: "missing", Token: "abc"}
	err := l.Release(ctx, guard)
	assert.ErrorIs(t, err, distributedlock.ErrLockNotFound)
}

// IsLockedがロック前はfalse、取得後はtrueを返すことを確認する。
func TestIsLocked(t *testing.T) {
	l := distributedlock.NewInMemoryLock()
	ctx := context.Background()

	locked, _ := l.IsLocked(ctx, "key1")
	assert.False(t, locked)

	_, _ = l.Acquire(ctx, "key1", time.Second)
	locked, _ = l.IsLocked(ctx, "key1")
	assert.True(t, locked)
}
