package distributedlock

import (
	"context"
	"database/sql"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// lockKeyがデフォルトプレフィックス「lock:」を付けてキーを返すことを確認する。
func TestPostgresLock_LockKey(t *testing.T) {
	l := NewPostgresLock(nil)
	assert.Equal(t, "lock:mykey", l.lockKey("mykey"))
}

// カスタムプレフィックスが設定された場合にlockKeyが正しいキーを返すことを確認する。
func TestPostgresLock_LockKey_CustomPrefix(t *testing.T) {
	l := NewPostgresLock(nil, WithPostgresLockPrefix("myapp:lock"))
	assert.Equal(t, "myapp:lock:mykey", l.lockKey("mykey"))
}

// PostgresLockのデフォルトプレフィックスが「lock」であることを確認する。
func TestPostgresLock_DefaultPrefix(t *testing.T) {
	l := NewPostgresLock(nil)
	assert.Equal(t, "lock", l.keyPrefix)
}

// 無効なPostgres URLからPostgresLockを生成するとエラーが返ることを確認する。
func TestNewPostgresLockFromURL_InvalidURL(t *testing.T) {
	// ドライバ未登録の場合 "unknown driver" エラーとなる
	_, err := NewPostgresLockFromURL("postgres://localhost:5432/testdb")
	require.Error(t, err)
}

// NewPostgresLockがnilのDBで構築でき、activeLocks が初期化されることを確認する。
func TestNewPostgresLock_WithDB(t *testing.T) {
	l := NewPostgresLock(nil)
	assert.NotNil(t, l)
	assert.Nil(t, l.db)
	assert.NotNil(t, l.activeLocks)
}

// PostgresLockがDistributedLockインターフェースを実装していることをコンパイル時に確認する。
func TestNewPostgresLock_ImplementsInterface(t *testing.T) {
	var _ DistributedLock = (*PostgresLock)(nil)
}

// ネストされたプレフィックスでlockKeyが正しく組み立てられることを確認する。
func TestPostgresLock_LockKey_NestedPrefix(t *testing.T) {
	l := NewPostgresLock(nil, WithPostgresLockPrefix("app:service:lock"))
	assert.Equal(t, "app:service:lock:resource-123", l.lockKey("resource-123"))
}

// コロンを含む特殊文字キーに対してlockKeyが正しく動作することを確認する。
func TestPostgresLock_LockKey_SpecialCharacters(t *testing.T) {
	l := NewPostgresLock(nil)
	assert.Equal(t, "lock:scheduler:job-123", l.lockKey("scheduler:job-123"))
}

// WithPostgresLockPrefixオプションでカスタムプレフィックスが設定されることを確認する。
func TestNewPostgresLockFromURL_CustomPrefix(t *testing.T) {
	l := NewPostgresLock(nil, WithPostgresLockPrefix("custom"))
	assert.Equal(t, "custom", l.keyPrefix)
}

// DBがnilの場合にAcquireがパニックすることを確認する。
func TestPostgresLock_Acquire_NilDB(t *testing.T) {
	l := NewPostgresLock(nil)
	assert.Panics(t, func() {
		_, _ = l.Acquire(context.Background(), "key1", 0)
	})
}

// 取得していないキーのReleaseがErrLockNotFoundを返すことを確認する。
func TestPostgresLock_Release_NotAcquired(t *testing.T) {
	l := NewPostgresLock(nil)
	guard := &LockGuard{Key: "missing", Token: "abc"}
	err := l.Release(context.Background(), guard)
	assert.ErrorIs(t, err, ErrLockNotFound)
}

// 新規作成したPostgresLockのactiveLocks マップが空で初期化されることを確認する。
func TestPostgresLock_ActiveLocks_Initialized(t *testing.T) {
	l := NewPostgresLock(nil)
	assert.Empty(t, l.activeLocks)
}

// トークン不一致の場合にReleaseがErrTokenMismatchを返しロックを保持し続けることを確認する。
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

// Postgres接続エラー時にAcquireがエラーを返すことを確認する。
func TestPostgresLock_Acquire_ConnectionError(t *testing.T) {
	l, cleanup := setupPostgresLock(t)
	defer cleanup()

	_, err := l.Acquire(context.Background(), "key1", 0)
	assert.Error(t, err)
}

// Postgres接続エラー時にIsLockedがエラーを返すことを確認する。
func TestPostgresLock_IsLocked_ConnectionError(t *testing.T) {
	l, cleanup := setupPostgresLock(t)
	defer cleanup()

	_, err := l.IsLocked(context.Background(), "key1")
	assert.Error(t, err)
}
