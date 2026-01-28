// Package k1s0cache provides Redis caching for the k1s0 framework.
//
// This package implements cache patterns:
//   - Get/Set/Delete operations
//   - Cache-aside pattern with GetOrSet
//   - Multiple serialization formats (JSON, Msgpack)
//   - Key prefixing for namespace isolation
//
// # Configuration
//
// Cache configuration follows k1s0 conventions:
//   - No environment variables (use config files)
//   - Secrets referenced via *_file suffix
//
// Example config.yaml:
//
//	cache:
//	  host: localhost
//	  port: 6379
//	  database: 0
//	  password_file: /var/run/secrets/k1s0/redis_password
//	  key_prefix: "myapp:"
//	  default_ttl: 5m
//	  serializer: json  # or msgpack
//
// # Usage
//
//	client, err := k1s0cache.NewClient(cacheConfig)
//	defer client.Close()
//
//	// Set value
//	err = client.Set(ctx, "user:123", user, 5*time.Minute)
//
//	// Get value
//	var user User
//	err = client.Get(ctx, "user:123", &user)
//
//	// Cache-aside pattern
//	user, err := k1s0cache.GetOrSet(ctx, client, "user:123", 5*time.Minute, func() (*User, error) {
//	    return repo.FindByID(ctx, "123")
//	})
package k1s0cache

import (
	"errors"
	"os"
	"strings"
	"time"
)

// CacheConfig holds cache configuration.
type CacheConfig struct {
	// Host is the Redis host.
	Host string

	// Port is the Redis port.
	Port int

	// Database is the Redis database number (0-15).
	Database int

	// Password is the Redis password (direct value, not recommended).
	Password string

	// PasswordFile is the path to a file containing the password (recommended).
	PasswordFile string

	// KeyPrefix is the prefix for all keys.
	KeyPrefix string

	// DefaultTTL is the default time-to-live for cache entries.
	DefaultTTL time.Duration

	// Serializer is the serializer type (json or msgpack).
	Serializer string

	// Pool is the connection pool configuration.
	Pool PoolConfig

	// TLS enables TLS connection.
	TLS bool

	// TLSSkipVerify skips TLS certificate verification (not recommended for production).
	TLSSkipVerify bool
}

// PoolConfig holds connection pool configuration.
type PoolConfig struct {
	// PoolSize is the maximum number of connections.
	// Default is 10.
	PoolSize int

	// MinIdleConns is the minimum number of idle connections.
	// Default is 5.
	MinIdleConns int

	// DialTimeout is the timeout for establishing new connections.
	// Default is 5 seconds.
	DialTimeout time.Duration

	// ReadTimeout is the timeout for read operations.
	// Default is 3 seconds.
	ReadTimeout time.Duration

	// WriteTimeout is the timeout for write operations.
	// Default is 3 seconds.
	WriteTimeout time.Duration

	// PoolTimeout is the timeout for getting a connection from the pool.
	// Default is 4 seconds.
	PoolTimeout time.Duration

	// MaxRetries is the maximum number of retries before giving up.
	// Default is 3.
	MaxRetries int
}

// DefaultPoolConfig returns a PoolConfig with default values.
func DefaultPoolConfig() PoolConfig {
	return PoolConfig{
		PoolSize:     10,
		MinIdleConns: 5,
		DialTimeout:  5 * time.Second,
		ReadTimeout:  3 * time.Second,
		WriteTimeout: 3 * time.Second,
		PoolTimeout:  4 * time.Second,
		MaxRetries:   3,
	}
}

// Validate validates the pool configuration and applies defaults.
func (c *PoolConfig) Validate() *PoolConfig {
	if c.PoolSize <= 0 {
		c.PoolSize = 10
	}
	if c.MinIdleConns <= 0 {
		c.MinIdleConns = 5
	}
	if c.MinIdleConns > c.PoolSize {
		c.MinIdleConns = c.PoolSize
	}
	if c.DialTimeout <= 0 {
		c.DialTimeout = 5 * time.Second
	}
	if c.ReadTimeout <= 0 {
		c.ReadTimeout = 3 * time.Second
	}
	if c.WriteTimeout <= 0 {
		c.WriteTimeout = 3 * time.Second
	}
	if c.PoolTimeout <= 0 {
		c.PoolTimeout = 4 * time.Second
	}
	if c.MaxRetries < 0 {
		c.MaxRetries = 3
	}
	return c
}

// DefaultCacheConfig returns a CacheConfig with default values.
func DefaultCacheConfig() *CacheConfig {
	return &CacheConfig{
		Host:       "localhost",
		Port:       6379,
		Database:   0,
		KeyPrefix:  "",
		DefaultTTL: 5 * time.Minute,
		Serializer: "json",
		Pool:       DefaultPoolConfig(),
	}
}

// Validate validates the cache configuration.
func (c *CacheConfig) Validate() error {
	if c.Host == "" {
		return errors.New("cache host is required")
	}
	if c.Port <= 0 || c.Port > 65535 {
		return errors.New("cache port must be between 1 and 65535")
	}
	if c.Database < 0 || c.Database > 15 {
		return errors.New("cache database must be between 0 and 15")
	}
	if c.Serializer != "" && c.Serializer != "json" && c.Serializer != "msgpack" {
		return errors.New("serializer must be 'json' or 'msgpack'")
	}

	c.Pool.Validate()
	return nil
}

// GetPassword returns the password, reading from file if necessary.
func (c *CacheConfig) GetPassword() (string, error) {
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
	return "", nil // No password is valid for Redis
}

// CacheConfigBuilder builds a CacheConfig.
type CacheConfigBuilder struct {
	config *CacheConfig
}

// NewCacheConfigBuilder creates a new CacheConfigBuilder.
func NewCacheConfigBuilder() *CacheConfigBuilder {
	return &CacheConfigBuilder{
		config: DefaultCacheConfig(),
	}
}

// Host sets the Redis host.
func (b *CacheConfigBuilder) Host(host string) *CacheConfigBuilder {
	b.config.Host = host
	return b
}

// Port sets the Redis port.
func (b *CacheConfigBuilder) Port(port int) *CacheConfigBuilder {
	b.config.Port = port
	return b
}

// Database sets the Redis database number.
func (b *CacheConfigBuilder) Database(db int) *CacheConfigBuilder {
	b.config.Database = db
	return b
}

// Password sets the Redis password.
func (b *CacheConfigBuilder) Password(password string) *CacheConfigBuilder {
	b.config.Password = password
	return b
}

// PasswordFile sets the path to the password file.
func (b *CacheConfigBuilder) PasswordFile(path string) *CacheConfigBuilder {
	b.config.PasswordFile = path
	return b
}

// KeyPrefix sets the key prefix.
func (b *CacheConfigBuilder) KeyPrefix(prefix string) *CacheConfigBuilder {
	b.config.KeyPrefix = prefix
	return b
}

// DefaultTTL sets the default TTL.
func (b *CacheConfigBuilder) DefaultTTL(ttl time.Duration) *CacheConfigBuilder {
	b.config.DefaultTTL = ttl
	return b
}

// Serializer sets the serializer type.
func (b *CacheConfigBuilder) Serializer(serializer string) *CacheConfigBuilder {
	b.config.Serializer = serializer
	return b
}

// PoolSize sets the pool size.
func (b *CacheConfigBuilder) PoolSize(size int) *CacheConfigBuilder {
	b.config.Pool.PoolSize = size
	return b
}

// MinIdleConns sets the minimum idle connections.
func (b *CacheConfigBuilder) MinIdleConns(min int) *CacheConfigBuilder {
	b.config.Pool.MinIdleConns = min
	return b
}

// TLS enables TLS connection.
func (b *CacheConfigBuilder) TLS(enabled bool) *CacheConfigBuilder {
	b.config.TLS = enabled
	return b
}

// Build creates the CacheConfig.
func (b *CacheConfigBuilder) Build() (*CacheConfig, error) {
	if err := b.config.Validate(); err != nil {
		return nil, err
	}
	return b.config, nil
}
