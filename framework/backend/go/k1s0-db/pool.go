package k1s0db

import (
	"context"
	"fmt"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/jackc/pgx/v5/pgxpool"
)

// Pool represents a database connection pool.
type Pool interface {
	// Exec executes a query that doesn't return rows.
	Exec(ctx context.Context, sql string, args ...any) (pgconn.CommandTag, error)

	// Query executes a query that returns rows.
	Query(ctx context.Context, sql string, args ...any) (pgx.Rows, error)

	// QueryRow executes a query that returns at most one row.
	QueryRow(ctx context.Context, sql string, args ...any) pgx.Row

	// Begin starts a transaction.
	Begin(ctx context.Context) (pgx.Tx, error)

	// BeginTx starts a transaction with options.
	BeginTx(ctx context.Context, txOptions pgx.TxOptions) (pgx.Tx, error)

	// Ping checks if the database is reachable.
	Ping(ctx context.Context) error

	// Close closes the pool.
	Close()

	// Stats returns pool statistics.
	Stats() *pgxpool.Stat
}

// ConnectionPool wraps a pgxpool.Pool.
type ConnectionPool struct {
	pool   *pgxpool.Pool
	config *DBConfig
}

// NewConnectionPool creates a new connection pool.
//
// Example:
//
//	config, _ := k1s0db.NewDBConfigBuilder().
//	    Host("localhost").
//	    Port(5432).
//	    Database("mydb").
//	    User("myuser").
//	    Password("secret").
//	    Build()
//
//	pool, err := k1s0db.NewConnectionPool(ctx, config)
//	if err != nil {
//	    log.Fatal(err)
//	}
//	defer pool.Close()
func NewConnectionPool(ctx context.Context, config *DBConfig) (*ConnectionPool, error) {
	if err := config.Validate(); err != nil {
		return nil, fmt.Errorf("invalid config: %w", err)
	}

	password, err := config.GetPassword()
	if err != nil {
		return nil, fmt.Errorf("failed to get password: %w", err)
	}

	// Build connection string
	connStr := fmt.Sprintf(
		"host=%s port=%d dbname=%s user=%s password=%s sslmode=%s",
		config.Host,
		config.Port,
		config.Database,
		config.User,
		password,
		config.SSLMode,
	)

	// Parse config
	poolConfig, err := pgxpool.ParseConfig(connStr)
	if err != nil {
		return nil, fmt.Errorf("failed to parse connection string: %w", err)
	}

	// Apply pool settings
	poolConfig.MaxConns = config.Pool.MaxConns
	poolConfig.MinConns = config.Pool.MinConns
	poolConfig.MaxConnLifetime = config.Pool.MaxConnLifetime
	poolConfig.MaxConnIdleTime = config.Pool.MaxConnIdleTime
	poolConfig.HealthCheckPeriod = config.Pool.HealthCheckPeriod

	// Create pool
	pool, err := pgxpool.NewWithConfig(ctx, poolConfig)
	if err != nil {
		return nil, fmt.Errorf("failed to create pool: %w", err)
	}

	// Verify connection
	if err := pool.Ping(ctx); err != nil {
		pool.Close()
		return nil, fmt.Errorf("failed to ping database: %w", err)
	}

	return &ConnectionPool{
		pool:   pool,
		config: config,
	}, nil
}

// Exec executes a query that doesn't return rows.
func (p *ConnectionPool) Exec(ctx context.Context, sql string, args ...any) (pgconn.CommandTag, error) {
	return p.pool.Exec(ctx, sql, args...)
}

// Query executes a query that returns rows.
func (p *ConnectionPool) Query(ctx context.Context, sql string, args ...any) (pgx.Rows, error) {
	return p.pool.Query(ctx, sql, args...)
}

// QueryRow executes a query that returns at most one row.
func (p *ConnectionPool) QueryRow(ctx context.Context, sql string, args ...any) pgx.Row {
	return p.pool.QueryRow(ctx, sql, args...)
}

// Begin starts a transaction.
func (p *ConnectionPool) Begin(ctx context.Context) (pgx.Tx, error) {
	return p.pool.Begin(ctx)
}

// BeginTx starts a transaction with options.
func (p *ConnectionPool) BeginTx(ctx context.Context, txOptions pgx.TxOptions) (pgx.Tx, error) {
	return p.pool.BeginTx(ctx, txOptions)
}

// Ping checks if the database is reachable.
func (p *ConnectionPool) Ping(ctx context.Context) error {
	return p.pool.Ping(ctx)
}

// Close closes the pool.
func (p *ConnectionPool) Close() {
	p.pool.Close()
}

// Stats returns pool statistics.
func (p *ConnectionPool) Stats() *pgxpool.Stat {
	return p.pool.Stat()
}

// Config returns the database configuration.
func (p *ConnectionPool) Config() *DBConfig {
	return p.config
}

// PoolStats holds simplified pool statistics.
type PoolStats struct {
	// AcquireCount is the total number of successful connection acquisitions.
	AcquireCount int64

	// AcquireDuration is the total duration of all acquisitions.
	AcquireDuration int64

	// AcquiredConns is the number of currently acquired connections.
	AcquiredConns int32

	// CanceledAcquireCount is the number of acquisitions canceled.
	CanceledAcquireCount int64

	// ConstructingConns is the number of connections being created.
	ConstructingConns int32

	// EmptyAcquireCount is the number of acquisitions that had to wait.
	EmptyAcquireCount int64

	// IdleConns is the number of idle connections.
	IdleConns int32

	// MaxConns is the maximum number of connections.
	MaxConns int32

	// TotalConns is the total number of connections.
	TotalConns int32
}

// GetStats returns simplified pool statistics.
func (p *ConnectionPool) GetStats() PoolStats {
	stat := p.pool.Stat()
	return PoolStats{
		AcquireCount:         stat.AcquireCount(),
		AcquireDuration:      stat.AcquireDuration().Nanoseconds(),
		AcquiredConns:        stat.AcquiredConns(),
		CanceledAcquireCount: stat.CanceledAcquireCount(),
		ConstructingConns:    stat.ConstructingConns(),
		EmptyAcquireCount:    stat.EmptyAcquireCount(),
		IdleConns:            stat.IdleConns(),
		MaxConns:             stat.MaxConns(),
		TotalConns:           stat.TotalConns(),
	}
}
