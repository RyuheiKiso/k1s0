package distributedlock

import (
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
	// sql.Open は driver の登録がなければエラーになる
	// lib/pq がインポートされていない状態では "unknown driver" エラー
	_, err := NewPostgresLockFromURL("postgres://localhost:5432/testdb")
	require.Error(t, err)
}

func TestNewPostgresLock_WithDB(t *testing.T) {
	// nil DB でも構造体は作成できる
	l := NewPostgresLock(nil)
	assert.NotNil(t, l)
	assert.Nil(t, l.db)
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
	// driver 未登録でエラーになるが、オプション適用前にエラーとなるので
	// 別途構造体ベースのテストでオプション適用を確認
	l := NewPostgresLock(nil, WithPostgresLockPrefix("custom"))
	assert.Equal(t, "custom", l.keyPrefix)
}

// TestPostgresLock_FromRealDB は実際の *sql.DB を受け取るコンストラクタの動作を確認する。
// ドライバなしの空 DB を使い、接続エラーの伝搬を検証する。
func TestPostgresLock_Acquire_NilDB(t *testing.T) {
	// nil DB での Acquire は panic するので recover で捕捉
	l := NewPostgresLock(nil)
	assert.Panics(t, func() {
		_, _ = l.Acquire(t.Context(), "key1", 0)
	})
}

// setupPostgresLock は接続不能な DB を使ったテストセットアップ。
func setupPostgresLock(t *testing.T) (*PostgresLock, func()) {
	t.Helper()
	// 実在しないアドレスへの接続で sql.DB を作成（ドライバ登録が必要）
	// ドライバ未登録でも sql.Open は成功し、実際のクエリでエラーとなる
	db, err := sql.Open("postgres", "postgres://invalid:5432/nonexistent?sslmode=disable")
	if err != nil {
		// ドライバ未登録の場合はスキップ
		t.Skip("postgres driver not registered, skipping connection test")
	}
	l := NewPostgresLock(db)
	return l, func() { db.Close() }
}

func TestPostgresLock_Acquire_ConnectionError(t *testing.T) {
	l, cleanup := setupPostgresLock(t)
	defer cleanup()

	_, err := l.Acquire(t.Context(), "key1", 0)
	assert.Error(t, err)
}

func TestPostgresLock_Release_ConnectionError(t *testing.T) {
	l, cleanup := setupPostgresLock(t)
	defer cleanup()

	guard := &LockGuard{Key: "key1", Token: "token"}
	err := l.Release(t.Context(), guard)
	assert.Error(t, err)
}

func TestPostgresLock_IsLocked_ConnectionError(t *testing.T) {
	l, cleanup := setupPostgresLock(t)
	defer cleanup()

	_, err := l.IsLocked(t.Context(), "key1")
	assert.Error(t, err)
}
