// Package k1s0db provides PostgreSQL database access for the k1s0 framework.
//
// This package implements Clean Architecture database patterns:
//   - Connection pool management with health checks
//   - Transaction management with automatic rollback
//   - Base repository for common CRUD operations
//   - Optional migration runner
//
// # Configuration
//
// Database configuration follows k1s0 conventions:
//   - No environment variables (use config files)
//   - Secrets referenced via *_file suffix
//
// Example config.yaml:
//
//	database:
//	  host: localhost
//	  port: 5432
//	  database: mydb
//	  user: myuser
//	  password_file: /var/run/secrets/k1s0/db_password
//	  pool:
//	    max_conns: 25
//	    min_conns: 5
//	    max_conn_lifetime: 1h
//	    max_conn_idle_time: 30m
//	    health_check_period: 1m
//
// # Usage
//
//	pool, err := k1s0db.NewConnectionPool(ctx, dbConfig)
//	defer pool.Close()
//
//	// Execute query
//	rows, err := pool.Query(ctx, "SELECT * FROM users WHERE id = $1", id)
//
//	// Transaction
//	txManager := k1s0db.NewTxManager(pool)
//	err := txManager.RunInTx(ctx, func(tx k1s0db.Tx) error {
//	    _, err := tx.Exec(ctx, "INSERT INTO users (id, name) VALUES ($1, $2)", id, name)
//	    return err
//	})
package k1s0db

import (
	"errors"
	"os"
	"strings"
	"time"
)

// DBConfig holds database configuration.
type DBConfig struct {
	// Host is the database host.
	Host string

	// Port is the database port.
	Port int

	// Database is the database name.
	Database string

	// User is the database user.
	User string

	// Password is the database password (direct value, not recommended).
	Password string

	// PasswordFile is the path to a file containing the password (recommended).
	PasswordFile string

	// SSLMode is the SSL mode (disable, require, verify-ca, verify-full).
	SSLMode string

	// Pool is the connection pool configuration.
	Pool PoolConfig
}

// PoolConfig holds connection pool configuration.
type PoolConfig struct {
	// MaxConns is the maximum number of connections in the pool.
	// Default is 25.
	MaxConns int32

	// MinConns is the minimum number of connections to keep open.
	// Default is 5.
	MinConns int32

	// MaxConnLifetime is the maximum lifetime of a connection.
	// Default is 1 hour.
	MaxConnLifetime time.Duration

	// MaxConnIdleTime is the maximum idle time of a connection.
	// Default is 30 minutes.
	MaxConnIdleTime time.Duration

	// HealthCheckPeriod is the interval between health checks.
	// Default is 1 minute.
	HealthCheckPeriod time.Duration
}

// DefaultPoolConfig returns a PoolConfig with default values.
func DefaultPoolConfig() PoolConfig {
	return PoolConfig{
		MaxConns:          25,
		MinConns:          5,
		MaxConnLifetime:   time.Hour,
		MaxConnIdleTime:   30 * time.Minute,
		HealthCheckPeriod: time.Minute,
	}
}

// Validate validates the pool configuration and applies defaults.
func (c *PoolConfig) Validate() *PoolConfig {
	if c.MaxConns <= 0 {
		c.MaxConns = 25
	}
	if c.MinConns <= 0 {
		c.MinConns = 5
	}
	if c.MinConns > c.MaxConns {
		c.MinConns = c.MaxConns
	}
	if c.MaxConnLifetime <= 0 {
		c.MaxConnLifetime = time.Hour
	}
	if c.MaxConnIdleTime <= 0 {
		c.MaxConnIdleTime = 30 * time.Minute
	}
	if c.HealthCheckPeriod <= 0 {
		c.HealthCheckPeriod = time.Minute
	}
	return c
}

// DefaultDBConfig returns a DBConfig with default values.
func DefaultDBConfig() *DBConfig {
	return &DBConfig{
		Host:     "localhost",
		Port:     5432,
		Database: "postgres",
		User:     "postgres",
		SSLMode:  "disable",
		Pool:     DefaultPoolConfig(),
	}
}

// Validate validates the database configuration.
func (c *DBConfig) Validate() error {
	if c.Host == "" {
		return errors.New("database host is required")
	}
	if c.Port <= 0 || c.Port > 65535 {
		return errors.New("database port must be between 1 and 65535")
	}
	if c.Database == "" {
		return errors.New("database name is required")
	}
	if c.User == "" {
		return errors.New("database user is required")
	}
	if c.Password == "" && c.PasswordFile == "" {
		return errors.New("database password or password_file is required")
	}

	c.Pool.Validate()
	return nil
}

// GetPassword returns the password, reading from file if necessary.
func (c *DBConfig) GetPassword() (string, error) {
	if c.Password != "" {
		return c.Password, nil
	}
	if c.PasswordFile != "" {
		data, err := os.ReadFile(c.PasswordFile)
		if err != nil {
			return "", err
		}
		return strings.TrimSpace(string(data)), nil
	}
	return "", errors.New("no password configured")
}

// DBConfigBuilder builds a DBConfig.
type DBConfigBuilder struct {
	config *DBConfig
}

// NewDBConfigBuilder creates a new DBConfigBuilder.
func NewDBConfigBuilder() *DBConfigBuilder {
	return &DBConfigBuilder{
		config: DefaultDBConfig(),
	}
}

// Host sets the database host.
func (b *DBConfigBuilder) Host(host string) *DBConfigBuilder {
	b.config.Host = host
	return b
}

// Port sets the database port.
func (b *DBConfigBuilder) Port(port int) *DBConfigBuilder {
	b.config.Port = port
	return b
}

// Database sets the database name.
func (b *DBConfigBuilder) Database(database string) *DBConfigBuilder {
	b.config.Database = database
	return b
}

// User sets the database user.
func (b *DBConfigBuilder) User(user string) *DBConfigBuilder {
	b.config.User = user
	return b
}

// Password sets the database password.
func (b *DBConfigBuilder) Password(password string) *DBConfigBuilder {
	b.config.Password = password
	return b
}

// PasswordFile sets the path to the password file.
func (b *DBConfigBuilder) PasswordFile(path string) *DBConfigBuilder {
	b.config.PasswordFile = path
	return b
}

// SSLMode sets the SSL mode.
func (b *DBConfigBuilder) SSLMode(mode string) *DBConfigBuilder {
	b.config.SSLMode = mode
	return b
}

// MaxConns sets the maximum number of connections.
func (b *DBConfigBuilder) MaxConns(max int32) *DBConfigBuilder {
	b.config.Pool.MaxConns = max
	return b
}

// MinConns sets the minimum number of connections.
func (b *DBConfigBuilder) MinConns(min int32) *DBConfigBuilder {
	b.config.Pool.MinConns = min
	return b
}

// MaxConnLifetime sets the maximum connection lifetime.
func (b *DBConfigBuilder) MaxConnLifetime(d time.Duration) *DBConfigBuilder {
	b.config.Pool.MaxConnLifetime = d
	return b
}

// MaxConnIdleTime sets the maximum connection idle time.
func (b *DBConfigBuilder) MaxConnIdleTime(d time.Duration) *DBConfigBuilder {
	b.config.Pool.MaxConnIdleTime = d
	return b
}

// HealthCheckPeriod sets the health check period.
func (b *DBConfigBuilder) HealthCheckPeriod(d time.Duration) *DBConfigBuilder {
	b.config.Pool.HealthCheckPeriod = d
	return b
}

// Build creates the DBConfig.
func (b *DBConfigBuilder) Build() (*DBConfig, error) {
	if err := b.config.Validate(); err != nil {
		return nil, err
	}
	return b.config, nil
}
