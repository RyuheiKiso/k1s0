package distributedlock

import (
	"context"
	"database/sql"
	"time"
)

// PostgresLock は PostgreSQL advisory lock を使用した分散ロック実装。
// pg_try_advisory_lock(hashtext(key)) で非ブロッキングにロックを取得し、
// pg_advisory_unlock(hashtext(key)) で解放する。
// advisory lock はセッションスコープのため、TTL はアプリケーション側で管理する。
type PostgresLock struct {
	db        *sql.DB
	keyPrefix string
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
		db:        db,
		keyPrefix: "lock",
	}
	for _, opt := range opts {
		opt(l)
	}
	return l
}

// NewPostgresLockFromURL は PostgreSQL 接続 URL から新しい PostgresLock を生成する。
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
// TTL は advisory lock では無視される（セッション終了まで保持）。
func (l *PostgresLock) Acquire(ctx context.Context, key string, _ time.Duration) (*LockGuard, error) {
	fullKey := l.lockKey(key)
	token := generateToken()

	var acquired bool
	err := l.db.QueryRowContext(ctx, "SELECT pg_try_advisory_lock(hashtext($1))", fullKey).Scan(&acquired)
	if err != nil {
		return nil, err
	}
	if !acquired {
		return nil, ErrAlreadyLocked
	}

	return &LockGuard{Key: key, Token: token}, nil
}

// Release は advisory lock を解放する。
func (l *PostgresLock) Release(ctx context.Context, guard *LockGuard) error {
	fullKey := l.lockKey(guard.Key)

	var released bool
	err := l.db.QueryRowContext(ctx, "SELECT pg_advisory_unlock(hashtext($1))", fullKey).Scan(&released)
	if err != nil {
		return err
	}
	if !released {
		return ErrLockNotFound
	}

	return nil
}

// IsLocked は advisory lock が保持されているか確認する。
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
