package persistence

import (
	"context"
	"fmt"

	"github.com/k1s0-platform/system-server-go-auth/internal/infra/config"
)

// DB はデータベース接続を表す。
// 本番実装では sqlx.DB をラップする。
type DB struct {
	dsn string
}

// NewDB はデータベース接続を確立する。
func NewDB(cfg config.DatabaseConfig) (*DB, error) {
	dsn := cfg.DSN()
	if dsn == "" {
		return nil, fmt.Errorf("database DSN is empty")
	}
	return &DB{dsn: dsn}, nil
}

// Healthy はデータベースへの接続を確認する。
func (db *DB) Healthy(ctx context.Context) error {
	// 本番実装では db.PingContext(ctx) を呼ぶ
	return nil
}

// Close はデータベース接続を閉じる。
func (db *DB) Close() error {
	return nil
}
