package distributedlock

import (
	"context"
	"database/sql"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestPostgresLock_LockKey(t *testing.T) {
	l := NewPostgresLock(nil)
	assert.Equal(t, "lock:mykey", l.lockKey("mykey"))
}

func TestPostgresLock_LockKey_CustomPrefix(t *testing.T) {
	l := NewPostgresLock(nil, WithPostgresLockPrefix("myapp:lock"))
	assert.Equal(t, "myapp:lock:mykey", l.lockKey("mykey"))
}

func TestPostgresLock_DefaultPrefix(t *testing.T) {
	l := NewPostgresLock(nil)
	assert.Equal(t, "lock", l.keyPrefix)
}

func TestNewPostgresLockFromURL_InvalidURL(t *testing.T) {
	// ドライバ未登録の場合 "unknown driver" エラーとなる
	_, err := NewPostgresLockFromURL("postgres://localhost:5432/testdb")
	require.Error(t, err)
}

func TestNewPostgresLock_WithDB(t *testing.T) {
	l := NewPostgresLock(nil)
	assert.NotNil(t, l)
	assert.Nil(t, l.db)
	assert.NotNil(t, l.activeLocks)
}

func TestNewPostgresLock_ImplementsInterface(t *testing.T) {
	var _ DistributedLock = (*PostgresLock)(nil)
}

func TestPostgresLock_LockKey_NestedPrefix(t *testing.T) {
	l := NewPostgresLock(nil, WithPostgresLockPrefix("app:service:lock"))
	assert.Equal(t, "app:service:lock:resource-123", l.lockKey("resource-123"))
}

func TestPostgresLock_LockKey_SpecialCharacters(t *testing.T) {
	l := NewPostgresLock(nil)
	assert.Equal(t, "lock:scheduler:job-123", l.lockKey("scheduler:job-123"))
}

func TestNewPostgresLockFromURL_CustomPrefix(t *testing.T) {
	l := NewPostgresLock(nil, WithPostgresLockPrefix("custom"))
	assert.Equal(t, "custom", l.keyPrefix)
}

func TestPostgresLock_Acquire_NilDB(t *testing.T) {
	l := NewPostgresLock(nil)
	assert.Panics(t, func() {
		_, _ = l.Acquire(context.Background(), "key1", 0)
	})
}

func TestPostgresLock_Release_NotAcquired(t *testing.T) {
	l := NewPostgresLock(nil)
	guard := &LockGuard{Key: "missing", Token: "abc"}
	err := l.Release(context.Background(), guard)
	assert.ErrorIs(t, err, ErrLockNotFound)
}

func TestPostgresLock_ActiveLocks_Initialized(t *testing.T) {
	l := NewPostgresLock(nil)
	assert.Empty(t, l.activeLocks)
}

func TestPostgresLock_Release_TokenMismatch(t *testing.T) {
	l := NewPostgresLock(nil)
	// activeLocks にエントリを直接設定してトークン検証をテスト
	l.activeLocks["lock:key1"] = activeLock{conn: nil, token: "correct-token"}

	wrongGuard := &LockGuard{Key: "key1", Token: "wrong-token"}
	err := l.Release(context.Background(), wrongGuard)
	assert.ErrorIs(t, err, ErrTokenMismatch)

	// トークン不一致の場合、ロックは解放されない
	assert.Contains(t, l.activeLocks, "lock:key1")
}

// setupPostgresLock は接続不能な DB を使ったテストセットアップ。
func setupPostgresLock(t *testing.T) (*PostgresLock, func()) {
	t.Helper()
	db, err := sql.Open("postgres", "postgres://invalid:5432/nonexistent?sslmode=disable")
	if err != nil {
		t.Skip("postgres driver not registered, skipping connection test")
	}
	l := NewPostgresLock(db)
	return l, func() { db.Close() }
}

func TestPostgresLock_Acquire_ConnectionError(t *testing.T) {
	l, cleanup := setupPostgresLock(t)
	defer cleanup()

	_, err := l.Acquire(context.Background(), "key1", 0)
	assert.Error(t, err)
}

func TestPostgresLock_IsLocked_ConnectionError(t *testing.T) {
	l, cleanup := setupPostgresLock(t)
	defer cleanup()

	_, err := l.IsLocked(context.Background(), "key1")
	assert.Error(t, err)
}
