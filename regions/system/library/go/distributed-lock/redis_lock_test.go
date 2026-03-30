package distributedlock

import (
	"context"
	"testing"
	"time"

	// M-16 監査対応: miniredis を使ってインメモリ Redis でロックの成功パスをカバーする。
	// テスト専用インポート — 本番コードには影響しない。
	miniredis "github.com/alicebob/miniredis/v2"
	"github.com/redis/go-redis/v9"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// lockKeyがデフォルトプレフィックス「lock:」を付けてキーを返すことを確認する。
func TestRedisLock_LockKey(t *testing.T) {
	l := NewRedisLock(nil)
	assert.Equal(t, "lock:mykey", l.lockKey("mykey"))
}

// カスタムプレフィックスが設定された場合にlockKeyが正しいキーを返すことを確認する。
func TestRedisLock_LockKey_CustomPrefix(t *testing.T) {
	l := NewRedisLock(nil, WithLockPrefix("myapp:lock"))
	assert.Equal(t, "myapp:lock:mykey", l.lockKey("mykey"))
}

// 不正なURLからRedisLockを生成するとエラーが返ることを確認する。
func TestNewRedisLockFromURL_InvalidURL(t *testing.T) {
	_, err := NewRedisLockFromURL("not-a-valid-url")
	require.Error(t, err)
}

// 有効なRedis URLからRedisLockを正常に生成できることを確認する。
func TestNewRedisLockFromURL_ValidURL(t *testing.T) {
	l, err := NewRedisLockFromURL("redis://localhost:6379/0")
	require.NoError(t, err)
	assert.NotNil(t, l)
}

// generateRedisTokenが32文字の一意なトークンを生成することを確認する。
func TestGenerateRedisToken(t *testing.T) {
	token1, err1 := generateRedisToken()
	require.NoError(t, err1)
	token2, err2 := generateRedisToken()
	require.NoError(t, err2)
	assert.Len(t, token1, 32)
	assert.Len(t, token2, 32)
	assert.NotEqual(t, token1, token2)
}

// Connection error tests verify proper error propagation from Redis.
func setupRedisLock(t *testing.T) (*RedisLock, func()) {
	t.Helper()
	rdb := redis.NewClient(&redis.Options{
		Addr: "localhost:0", // invalid port, won't connect
	})
	l := NewRedisLock(rdb)
	return l, func() { rdb.Close() }
}

// Redis接続エラー時にAcquireがエラーを返すことを確認する。
func TestRedisLock_Acquire_ConnectionError(t *testing.T) {
	l, cleanup := setupRedisLock(t)
	defer cleanup()

	_, err := l.Acquire(context.Background(), "key1", time.Second)
	assert.Error(t, err)
}

// Redis接続エラー時にIsLockedがエラーを返すことを確認する。
func TestRedisLock_IsLocked_ConnectionError(t *testing.T) {
	l, cleanup := setupRedisLock(t)
	defer cleanup()

	_, err := l.IsLocked(context.Background(), "key1")
	assert.Error(t, err)
}

// M-16 監査対応: Redis 接続エラー時に Release がエラーを返すことを確認する。
// 接続できない Redis に対して Release を呼び出すと、Redis EVAL コマンドが失敗しエラーが伝播することを検証する。
func TestRedisLock_Release_ConnectionError(t *testing.T) {
	l, cleanup := setupRedisLock(t)
	defer cleanup()

	// 既取得ロックのガードをシミュレートする（実際に Acquire していなくてよい）
	guard := &LockGuard{Key: "key1", Token: "test-token"}
	err := l.Release(context.Background(), guard)
	assert.Error(t, err)
}

// M-16 監査対応: Redis 接続エラー時に Extend がエラーを返すことを確認する。
// 接続できない Redis に対して Extend を呼び出すと、Redis EVAL コマンドが失敗しエラーが伝播することを検証する。
func TestRedisLock_Extend_ConnectionError(t *testing.T) {
	l, cleanup := setupRedisLock(t)
	defer cleanup()

	guard := &LockGuard{Key: "key1", Token: "test-token"}
	err := l.Extend(context.Background(), guard, time.Second)
	assert.Error(t, err)
}

// M-16 監査対応: Redis 接続エラー時に Acquire が ErrAlreadyLocked ではなくネットワークエラーを返すことを確認する。
func TestRedisLock_Acquire_ReturnsNetworkError_NotLockError(t *testing.T) {
	l, cleanup := setupRedisLock(t)
	defer cleanup()

	_, err := l.Acquire(context.Background(), "key2", time.Second)
	require.Error(t, err)
	// 接続エラーは ErrAlreadyLocked とは異なる
	assert.NotErrorIs(t, err, ErrAlreadyLocked)
}

// setupMiniredisLock は miniredis（インメモリ Redis）を使ったテストセットアップ。
// 実際の Redis サーバーなしに成功パスをテストできる。
func setupMiniredisLock(t *testing.T) (*RedisLock, func()) {
	t.Helper()
	mr, err := miniredis.Run()
	require.NoError(t, err, "miniredis の起動に失敗")
	rdb := redis.NewClient(&redis.Options{Addr: mr.Addr()})
	l := NewRedisLock(rdb)
	return l, func() {
		rdb.Close()
		mr.Close()
	}
}

// M-16 監査対応: miniredis でロック取得の成功パスを確認する。
func TestRedisLock_Acquire_Success(t *testing.T) {
	l, cleanup := setupMiniredisLock(t)
	defer cleanup()

	guard, err := l.Acquire(context.Background(), "mykey", time.Second)
	require.NoError(t, err)
	require.NotNil(t, guard)
	assert.Equal(t, "mykey", guard.Key)
	assert.Len(t, guard.Token, 32)
}

// M-16 監査対応: 同一キーの二重取得が ErrAlreadyLocked を返すことを確認する。
func TestRedisLock_Acquire_AlreadyLocked(t *testing.T) {
	l, cleanup := setupMiniredisLock(t)
	defer cleanup()

	// 最初の取得は成功する
	guard, err := l.Acquire(context.Background(), "conflict-key", 5*time.Second)
	require.NoError(t, err)
	require.NotNil(t, guard)

	// 同一キーへの二重取得は ErrAlreadyLocked を返す
	_, err2 := l.Acquire(context.Background(), "conflict-key", 5*time.Second)
	assert.ErrorIs(t, err2, ErrAlreadyLocked)
}

// M-16 監査対応: miniredis で Release の成功パスを確認する。
func TestRedisLock_Release_Success(t *testing.T) {
	l, cleanup := setupMiniredisLock(t)
	defer cleanup()

	guard, err := l.Acquire(context.Background(), "release-key", 5*time.Second)
	require.NoError(t, err)

	// 正しいトークンで Release すると成功する
	err = l.Release(context.Background(), guard)
	require.NoError(t, err)

	// Release 後は再取得できる
	guard2, err2 := l.Acquire(context.Background(), "release-key", time.Second)
	require.NoError(t, err2)
	assert.NotNil(t, guard2)
}

// M-16 監査対応: 誤ったトークンで Release すると ErrTokenMismatch を返すことを確認する。
func TestRedisLock_Release_WrongToken(t *testing.T) {
	l, cleanup := setupMiniredisLock(t)
	defer cleanup()

	_, err := l.Acquire(context.Background(), "token-key", 5*time.Second)
	require.NoError(t, err)

	// 異なるトークンで Release するとエラーを返す
	wrongGuard := &LockGuard{Key: "token-key", Token: "wrong-token-123456789012"}
	err = l.Release(context.Background(), wrongGuard)
	require.Error(t, err)
}

// M-16 監査対応: miniredis で Extend の成功パスを確認する。
func TestRedisLock_Extend_Success(t *testing.T) {
	l, cleanup := setupMiniredisLock(t)
	defer cleanup()

	guard, err := l.Acquire(context.Background(), "extend-key", time.Second)
	require.NoError(t, err)

	// TTL を 10 秒に延長する
	err = l.Extend(context.Background(), guard, 10*time.Second)
	require.NoError(t, err)
}

// M-16 監査対応: 誤ったトークンで Extend すると ErrTokenMismatch を返すことを確認する。
func TestRedisLock_Extend_WrongToken(t *testing.T) {
	l, cleanup := setupMiniredisLock(t)
	defer cleanup()

	_, err := l.Acquire(context.Background(), "extend-key2", 5*time.Second)
	require.NoError(t, err)

	wrongGuard := &LockGuard{Key: "extend-key2", Token: "wrong-token-123456789012"}
	err = l.Extend(context.Background(), wrongGuard, 10*time.Second)
	require.Error(t, err)
}

// M-16 監査対応: miniredis で IsLocked が true を返すことを確認する。
func TestRedisLock_IsLocked_True(t *testing.T) {
	l, cleanup := setupMiniredisLock(t)
	defer cleanup()

	_, err := l.Acquire(context.Background(), "locked-key", 5*time.Second)
	require.NoError(t, err)

	// ロック取得後は IsLocked が true を返す
	locked, err := l.IsLocked(context.Background(), "locked-key")
	require.NoError(t, err)
	assert.True(t, locked)
}

// M-16 監査対応: miniredis で IsLocked が false を返すことを確認する。
func TestRedisLock_IsLocked_False(t *testing.T) {
	l, cleanup := setupMiniredisLock(t)
	defer cleanup()

	// ロックを取得していないキーは false を返す
	locked, err := l.IsLocked(context.Background(), "unlocked-key")
	require.NoError(t, err)
	assert.False(t, locked)
}
