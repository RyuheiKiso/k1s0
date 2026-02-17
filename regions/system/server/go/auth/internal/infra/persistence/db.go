package persistence

import (
	"context"
	"fmt"

	"github.com/jmoiron/sqlx"
	_ "github.com/lib/pq"
	"github.com/k1s0-platform/system-server-go-auth/internal/infra/config"
)

// DB はデータベース接続を表す。
type DB struct {
	conn *sqlx.DB
}

// NewDB はデータベース接続を確立する。
func NewDB(cfg config.DatabaseConfig) (*DB, error) {
	dsn := cfg.DSN()
	if dsn == "" {
		return nil, fmt.Errorf("database DSN is empty")
	}

	conn, err := sqlx.Connect("postgres", dsn)
	if err != nil {
		return nil, fmt.Errorf("failed to connect to database: %w", err)
	}

	return &DB{conn: conn}, nil
}

// Conn は内部の sqlx.DB を返す。
func (db *DB) Conn() *sqlx.DB {
	return db.conn
}

// Healthy はデータベースへの接続を確認する。
func (db *DB) Healthy(ctx context.Context) error {
	return db.conn.PingContext(ctx)
}

// Close はデータベース接続を閉じる。
func (db *DB) Close() error {
	return db.conn.Close()
}
