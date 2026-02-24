package health

import (
	"context"
	"database/sql"
	"fmt"
	"time"
)

// PostgresHealthCheck は PostgreSQL のヘルスを確認する。
type PostgresHealthCheck struct {
	name    string
	db      *sql.DB
	timeout time.Duration
}

// PostgresHealthCheckOption は PostgresHealthCheck の設定オプション。
type PostgresHealthCheckOption func(*PostgresHealthCheck)

// WithPostgresTimeout はタイムアウトを設定する。
func WithPostgresTimeout(d time.Duration) PostgresHealthCheckOption {
	return func(h *PostgresHealthCheck) {
		h.timeout = d
	}
}

// WithPostgresName はヘルスチェック名を設定する。
func WithPostgresName(name string) PostgresHealthCheckOption {
	return func(h *PostgresHealthCheck) {
		h.name = name
	}
}

// NewPostgresHealthCheck は新しい PostgresHealthCheck を生成する。
func NewPostgresHealthCheck(db *sql.DB, opts ...PostgresHealthCheckOption) *PostgresHealthCheck {
	h := &PostgresHealthCheck{
		name:    "postgres",
		db:      db,
		timeout: 5 * time.Second,
	}
	for _, opt := range opts {
		opt(h)
	}
	return h
}

// Name はヘルスチェック名を返す。
func (h *PostgresHealthCheck) Name() string {
	return h.name
}

// Check は PostgreSQL に対して ping を実行する。
func (h *PostgresHealthCheck) Check(ctx context.Context) error {
	checkCtx, cancel := context.WithTimeout(ctx, h.timeout)
	defer cancel()

	if err := h.db.PingContext(checkCtx); err != nil {
		return fmt.Errorf("postgres health check failed: %w", err)
	}
	return nil
}
