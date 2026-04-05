package distributedlock

import (
	"context"
	"database/sql"
	"testing"
	"time"

	// M-16 監査対応: postgres ドライバを登録して接続エラーパスのカバレッジを向上させる。
	// テスト専用インポート — 本番コードには影響しない。
	sqlmock "github.com/DATA-DOG/go-sqlmock"
	_ "github.com/lib/pq"
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

// NewPostgresLockFromURL は postgres ドライバが登録されていれば sql.Open が成功し
// 非 nil の PostgresLock を返すことを確認する。
// 実際の DB 接続は遅延して確立されるため、接続先が無効でも初期化は成功する。
func TestNewPostgresLockFromURL_ValidURL(t *testing.T) {
	l, err := NewPostgresLockFromURL("postgres://localhost:5432/testdb")
	require.NoError(t, err)
	assert.NotNil(t, l)
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

// MED-05 監査対応: nil DB の場合はパニックではなくエラーを返すことを確認する。
func TestPostgresLock_Acquire_NilDB(t *testing.T) {
	l := NewPostgresLock(nil)
	_, err := l.Acquire(context.Background(), "key1", 0)
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "db が初期化されていません")
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

// M-16 監査対応: sqlmock を使って Acquire の成功パスをカバーする。
// pg_try_advisory_lock が true を返す場合に LockGuard が返ることを確認する。
func TestPostgresLock_Acquire_Success(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	// pg_try_advisory_lock が true（ロック取得成功）を返すよう設定する
	mock.ExpectQuery("SELECT pg_try_advisory_lock").
		WillReturnRows(sqlmock.NewRows([]string{"pg_try_advisory_lock"}).AddRow(true))
	// Release 時の pg_advisory_unlock も設定する
	mock.ExpectQuery("SELECT pg_advisory_unlock").
		WillReturnRows(sqlmock.NewRows([]string{"pg_advisory_unlock"}).AddRow(true))

	l := NewPostgresLock(db)
	guard, acquireErr := l.Acquire(context.Background(), "mykey", time.Second)
	require.NoError(t, acquireErr)
	require.NotNil(t, guard)
	assert.Equal(t, "mykey", guard.Key)

	// Acquire で取得した guard を Release して後処理する
	require.NoError(t, l.Release(context.Background(), guard))
	assert.NoError(t, mock.ExpectationsWereMet())
}

// M-16 監査対応: pg_try_advisory_lock が false を返した場合に ErrAlreadyLocked が返ることを確認する。
func TestPostgresLock_Acquire_AlreadyLocked(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	// pg_try_advisory_lock が false（ロック取得失敗）を返すよう設定する
	mock.ExpectQuery("SELECT pg_try_advisory_lock").
		WillReturnRows(sqlmock.NewRows([]string{"pg_try_advisory_lock"}).AddRow(false))

	l := NewPostgresLock(db)
	_, acquireErr := l.Acquire(context.Background(), "mykey", time.Second)
	assert.ErrorIs(t, acquireErr, ErrAlreadyLocked)
	assert.NoError(t, mock.ExpectationsWereMet())
}

// M-16 監査対応: QueryRowContext がエラーを返す場合に Acquire がエラーを伝播することを確認する。
func TestPostgresLock_Acquire_QueryError(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	// クエリがエラーを返すよう設定する
	mock.ExpectQuery("SELECT pg_try_advisory_lock").
		WillReturnError(sql.ErrConnDone)

	l := NewPostgresLock(db)
	_, acquireErr := l.Acquire(context.Background(), "mykey", time.Second)
	require.Error(t, acquireErr)
	assert.NoError(t, mock.ExpectationsWereMet())
}

// M-16 監査対応: pg_advisory_unlock が false を返す場合に Release が ErrLockNotFound を返すことを確認する。
func TestPostgresLock_Release_UnlockFalse(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	// Acquire: ロック取得成功
	mock.ExpectQuery("SELECT pg_try_advisory_lock").
		WillReturnRows(sqlmock.NewRows([]string{"pg_try_advisory_lock"}).AddRow(true))
	// Release: unlock が false（DB 側でロックが既に解放済み）
	mock.ExpectQuery("SELECT pg_advisory_unlock").
		WillReturnRows(sqlmock.NewRows([]string{"pg_advisory_unlock"}).AddRow(false))

	l := NewPostgresLock(db)
	guard, err := l.Acquire(context.Background(), "mykey", time.Second)
	require.NoError(t, err)

	releaseErr := l.Release(context.Background(), guard)
	assert.ErrorIs(t, releaseErr, ErrLockNotFound)
	assert.NoError(t, mock.ExpectationsWereMet())
}

// M-16 監査対応: IsLocked が true を返す場合を sqlmock で確認する。
func TestPostgresLock_IsLocked_True(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	mock.ExpectQuery("SELECT EXISTS").
		WillReturnRows(sqlmock.NewRows([]string{"exists"}).AddRow(true))

	l := NewPostgresLock(db)
	locked, isErr := l.IsLocked(context.Background(), "mykey")
	require.NoError(t, isErr)
	assert.True(t, locked)
	assert.NoError(t, mock.ExpectationsWereMet())
}

// M-16 監査対応: IsLocked が false を返す場合を sqlmock で確認する。
func TestPostgresLock_IsLocked_False(t *testing.T) {
	db, mock, err := sqlmock.New()
	require.NoError(t, err)
	defer db.Close()

	mock.ExpectQuery("SELECT EXISTS").
		WillReturnRows(sqlmock.NewRows([]string{"exists"}).AddRow(false))

	l := NewPostgresLock(db)
	locked, isErr := l.IsLocked(context.Background(), "mykey")
	require.NoError(t, isErr)
	assert.False(t, locked)
	assert.NoError(t, mock.ExpectationsWereMet())
}
