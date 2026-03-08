package distributedlock

import (
	"context"
	"database/sql"
	"sync"
	"time"
)

// PostgresLock は PostgreSQL advisory lock を使用した分散ロック実装。
// pg_try_advisory_lock(hashtext(key)) で非ブロッキングにロックを取得し、
// pg_advisory_unlock(hashtext(key)) で解放する。
// advisory lock はセッション（コネクション）スコープのため、Acquire から Release まで
// 同一コネクションを保持する。TTL はアプリケーション側で管理する。
//
// 注意: advisory lock はコネクション単位で保持されるため、Acquire で取得した
// コネクションは Release まで占有される。高頻度で使用する場合は接続プールの
// サイズに注意すること。
type PostgresLock struct {
	db        *sql.DB
	keyPrefix string
	mu        sync.Mutex
	// activeLocks は Acquire で取得したコネクションとトークンをキー別に保持する。
	// Release 時に同一コネクションで pg_advisory_unlock を実行し、トークンを検証するために必要。
	activeLocks map[string]activeLock
}

type activeLock struct {
	conn  *sql.Conn
	token string
}

// PostgresLockOption は PostgresLock の設定オプション。
type PostgresLockOption func(*PostgresLock)

// WithPostgresLockPrefix はロックキーのプレフィックスを設定する。
func WithPostgresLockPrefix(prefix string) PostgresLockOption {
	return func(l *PostgresLock) {
		l.keyPrefix = prefix
	}
}

// NewPostgresLock は *sql.DB から新しい PostgresLock を生成する。
func NewPostgresLock(db *sql.DB, opts ...PostgresLockOption) *PostgresLock {
	l := &PostgresLock{
		db:          db,
		keyPrefix:   "lock",
		activeLocks: make(map[string]activeLock),
	}
	for _, opt := range opts {
		opt(l)
	}
	return l
}

// NewPostgresLockFromURL は PostgreSQL 接続 URL から新しい PostgresLock を生成する。
// 使用するには postgres ドライバ（github.com/lib/pq など）が登録されている必要がある。
func NewPostgresLockFromURL(url string, opts ...PostgresLockOption) (*PostgresLock, error) {
	db, err := sql.Open("postgres", url)
	if err != nil {
		return nil, err
	}
	db.SetMaxOpenConns(10)
	db.SetMaxIdleConns(5)
	db.SetConnMaxLifetime(5 * time.Minute)
	return NewPostgresLock(db, opts...), nil
}

func (l *PostgresLock) lockKey(key string) string {
	return l.keyPrefix + ":" + key
}

// Acquire は advisory lock で非ブロッキングにロックを取得する。
// TTL は advisory lock では無視される（コネクション終了まで保持）。
// 取得したコネクションは Release まで保持される。
func (l *PostgresLock) Acquire(ctx context.Context, key string, _ time.Duration) (*LockGuard, error) {
	fullKey := l.lockKey(key)
	token := generateToken()

	// 専用コネクションを取得し、advisory lock をそのコネクション上で保持する
	conn, err := l.db.Conn(ctx)
	if err != nil {
		return nil, err
	}

	var acquired bool
	err = conn.QueryRowContext(ctx, "SELECT pg_try_advisory_lock(hashtext($1))", fullKey).Scan(&acquired)
	if err != nil {
		conn.Close()
		return nil, err
	}
	if !acquired {
		conn.Close()
		return nil, ErrAlreadyLocked
	}

	// ロック取得に成功したら、Release 用にコネクションとトークンを保持する
	l.mu.Lock()
	l.activeLocks[fullKey] = activeLock{conn: conn, token: token}
	l.mu.Unlock()

	return &LockGuard{Key: key, Token: token}, nil
}

// Release は advisory lock を解放し、保持していたコネクションを返却する。
func (l *PostgresLock) Release(ctx context.Context, guard *LockGuard) error {
	fullKey := l.lockKey(guard.Key)

	l.mu.Lock()
	entry, ok := l.activeLocks[fullKey]
	if !ok {
		l.mu.Unlock()
		return ErrLockNotFound
	}
	if entry.token != guard.Token {
		l.mu.Unlock()
		return ErrTokenMismatch
	}
	delete(l.activeLocks, fullKey)
	l.mu.Unlock()

	defer entry.conn.Close()

	var released bool
	err := entry.conn.QueryRowContext(ctx, "SELECT pg_advisory_unlock(hashtext($1))", fullKey).Scan(&released)
	if err != nil {
		return err
	}
	if !released {
		return ErrLockNotFound
	}

	return nil
}

// IsLocked は advisory lock が保持されているか確認する。
// この確認は任意のコネクションで実行可能（pg_locks ビューの参照のため）。
func (l *PostgresLock) IsLocked(ctx context.Context, key string) (bool, error) {
	fullKey := l.lockKey(key)

	var exists bool
	err := l.db.QueryRowContext(
		ctx,
		"SELECT EXISTS(SELECT 1 FROM pg_locks WHERE locktype = 'advisory' AND classid = hashtext($1)::int)",
		fullKey,
	).Scan(&exists)
	if err != nil {
		return false, err
	}

	return exists, nil
}
