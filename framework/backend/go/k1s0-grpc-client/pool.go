package k1s0grpcclient

import (
	"context"
	"errors"
	"sync"
	"sync/atomic"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/connectivity"
)

// ErrPoolClosed is returned when the pool is closed.
var ErrPoolClosed = errors.New("connection pool is closed")

// ErrPoolExhausted is returned when the pool has no available connections.
var ErrPoolExhausted = errors.New("connection pool exhausted")

// PoolConfig holds connection pool configuration.
type PoolConfig struct {
	// MaxSize is the maximum number of connections in the pool.
	// Default is 10.
	MaxSize int

	// MinSize is the minimum number of connections to keep.
	// Default is 2.
	MinSize int

	// MaxIdleTime is the maximum time a connection can be idle before being closed.
	// Default is 30 minutes.
	MaxIdleTime time.Duration

	// HealthCheckInterval is the interval between health checks.
	// Default is 30 seconds.
	HealthCheckInterval time.Duration

	// WaitTimeout is the maximum time to wait for a connection.
	// Default is 5 seconds.
	WaitTimeout time.Duration
}

// DefaultPoolConfig returns a PoolConfig with default values.
func DefaultPoolConfig() *PoolConfig {
	return &PoolConfig{
		MaxSize:             10,
		MinSize:             2,
		MaxIdleTime:         30 * time.Minute,
		HealthCheckInterval: 30 * time.Second,
		WaitTimeout:         5 * time.Second,
	}
}

// poolConn wraps a connection with metadata.
type poolConn struct {
	conn      *grpc.ClientConn
	lastUsed  time.Time
	inUse     bool
	unhealthy bool
}

// ConnectionPool manages a pool of gRPC connections.
type ConnectionPool struct {
	client   *Client
	target   string
	config   *PoolConfig
	conns    []*poolConn
	mu       sync.Mutex
	closed   bool
	waitCh   chan struct{}
	acquired int64
	released int64
	created  int64
	failed   int64
}

// NewConnectionPool creates a new connection pool.
//
// Example:
//
//	poolConfig := k1s0grpcclient.DefaultPoolConfig()
//	poolConfig.MaxSize = 20
//
//	pool, err := k1s0grpcclient.NewConnectionPool(client, "user-service:50051", poolConfig)
//	if err != nil {
//	    log.Fatal(err)
//	}
//	defer pool.Close()
//
//	conn, release, err := pool.Acquire(ctx)
//	if err != nil {
//	    log.Fatal(err)
//	}
//	defer release()
//
//	userClient := pb.NewUserServiceClient(conn)
func NewConnectionPool(client *Client, target string, config *PoolConfig) (*ConnectionPool, error) {
	if config == nil {
		config = DefaultPoolConfig()
	}

	if config.MaxSize <= 0 {
		config.MaxSize = 10
	}
	if config.MinSize <= 0 {
		config.MinSize = 2
	}
	if config.MinSize > config.MaxSize {
		config.MinSize = config.MaxSize
	}

	pool := &ConnectionPool{
		client: client,
		target: target,
		config: config,
		conns:  make([]*poolConn, 0, config.MaxSize),
		waitCh: make(chan struct{}, config.MaxSize),
	}

	// Pre-create minimum connections
	ctx := context.Background()
	for i := 0; i < config.MinSize; i++ {
		if err := pool.createConnection(ctx); err != nil {
			// Log error but continue
			atomic.AddInt64(&pool.failed, 1)
		}
	}

	// Start health check goroutine
	if config.HealthCheckInterval > 0 {
		go pool.healthCheckLoop()
	}

	return pool, nil
}

// createConnection creates a new connection and adds it to the pool.
func (p *ConnectionPool) createConnection(ctx context.Context) error {
	conn, err := p.client.Dial(ctx, p.target)
	if err != nil {
		return err
	}

	p.conns = append(p.conns, &poolConn{
		conn:     conn,
		lastUsed: time.Now(),
		inUse:    false,
	})
	atomic.AddInt64(&p.created, 1)

	return nil
}

// Acquire gets a connection from the pool.
// Returns the connection and a release function that must be called when done.
func (p *ConnectionPool) Acquire(ctx context.Context) (*grpc.ClientConn, func(), error) {
	p.mu.Lock()

	if p.closed {
		p.mu.Unlock()
		return nil, nil, ErrPoolClosed
	}

	// Try to find an available connection
	for _, pc := range p.conns {
		if !pc.inUse && !pc.unhealthy {
			pc.inUse = true
			pc.lastUsed = time.Now()
			atomic.AddInt64(&p.acquired, 1)
			p.mu.Unlock()

			release := func() {
				p.release(pc)
			}
			return pc.conn, release, nil
		}
	}

	// No available connection, try to create a new one
	if len(p.conns) < p.config.MaxSize {
		if err := p.createConnection(ctx); err == nil {
			pc := p.conns[len(p.conns)-1]
			pc.inUse = true
			atomic.AddInt64(&p.acquired, 1)
			p.mu.Unlock()

			release := func() {
				p.release(pc)
			}
			return pc.conn, release, nil
		}
		atomic.AddInt64(&p.failed, 1)
	}

	// Pool is at capacity, wait for a connection
	p.mu.Unlock()

	select {
	case <-ctx.Done():
		return nil, nil, ctx.Err()
	case <-time.After(p.config.WaitTimeout):
		return nil, nil, ErrPoolExhausted
	case <-p.waitCh:
		// A connection became available, try again
		return p.Acquire(ctx)
	}
}

// release returns a connection to the pool.
func (p *ConnectionPool) release(pc *poolConn) {
	p.mu.Lock()
	defer p.mu.Unlock()

	pc.inUse = false
	pc.lastUsed = time.Now()
	atomic.AddInt64(&p.released, 1)

	// Notify waiters
	select {
	case p.waitCh <- struct{}{}:
	default:
	}
}

// healthCheckLoop periodically checks connection health.
func (p *ConnectionPool) healthCheckLoop() {
	ticker := time.NewTicker(p.config.HealthCheckInterval)
	defer ticker.Stop()

	for range ticker.C {
		p.mu.Lock()
		if p.closed {
			p.mu.Unlock()
			return
		}

		for _, pc := range p.conns {
			if pc.inUse {
				continue
			}

			// Check connection state
			state := pc.conn.GetState()
			if state == connectivity.TransientFailure || state == connectivity.Shutdown {
				pc.unhealthy = true
			} else {
				pc.unhealthy = false
			}

			// Close idle connections exceeding max idle time
			if time.Since(pc.lastUsed) > p.config.MaxIdleTime && len(p.conns) > p.config.MinSize {
				pc.unhealthy = true
			}
		}

		// Remove unhealthy connections
		healthy := p.conns[:0]
		for _, pc := range p.conns {
			if pc.unhealthy && !pc.inUse {
				pc.conn.Close()
			} else {
				healthy = append(healthy, pc)
			}
		}
		p.conns = healthy

		p.mu.Unlock()
	}
}

// Close closes all connections in the pool.
func (p *ConnectionPool) Close() error {
	p.mu.Lock()
	defer p.mu.Unlock()

	if p.closed {
		return nil
	}

	p.closed = true

	var lastErr error
	for _, pc := range p.conns {
		if err := pc.conn.Close(); err != nil {
			lastErr = err
		}
	}
	p.conns = nil

	return lastErr
}

// PoolStats holds pool statistics.
type PoolStats struct {
	// Size is the current number of connections.
	Size int

	// InUse is the number of connections in use.
	InUse int

	// Available is the number of available connections.
	Available int

	// Unhealthy is the number of unhealthy connections.
	Unhealthy int

	// Acquired is the total number of connections acquired.
	Acquired int64

	// Released is the total number of connections released.
	Released int64

	// Created is the total number of connections created.
	Created int64

	// Failed is the total number of connection creation failures.
	Failed int64
}

// Stats returns pool statistics.
func (p *ConnectionPool) Stats() PoolStats {
	p.mu.Lock()
	defer p.mu.Unlock()

	stats := PoolStats{
		Size:     len(p.conns),
		Acquired: atomic.LoadInt64(&p.acquired),
		Released: atomic.LoadInt64(&p.released),
		Created:  atomic.LoadInt64(&p.created),
		Failed:   atomic.LoadInt64(&p.failed),
	}

	for _, pc := range p.conns {
		if pc.inUse {
			stats.InUse++
		} else if pc.unhealthy {
			stats.Unhealthy++
		} else {
			stats.Available++
		}
	}

	return stats
}

// Target returns the target address.
func (p *ConnectionPool) Target() string {
	return p.target
}

// ConnectionPoolGroup manages multiple connection pools.
type ConnectionPoolGroup struct {
	client *Client
	config *PoolConfig
	pools  sync.Map // map[string]*ConnectionPool
}

// NewConnectionPoolGroup creates a new connection pool group.
func NewConnectionPoolGroup(client *Client, config *PoolConfig) *ConnectionPoolGroup {
	return &ConnectionPoolGroup{
		client: client,
		config: config,
	}
}

// Get returns the connection pool for the given target.
// Creates a new pool if one doesn't exist.
func (g *ConnectionPoolGroup) Get(target string) (*ConnectionPool, error) {
	if pool, ok := g.pools.Load(target); ok {
		return pool.(*ConnectionPool), nil
	}

	newPool, err := NewConnectionPool(g.client, target, g.config)
	if err != nil {
		return nil, err
	}

	actual, loaded := g.pools.LoadOrStore(target, newPool)
	if loaded {
		// Another goroutine created the pool, close our new one
		newPool.Close()
	}

	return actual.(*ConnectionPool), nil
}

// Acquire gets a connection from the pool for the given target.
func (g *ConnectionPoolGroup) Acquire(ctx context.Context, target string) (*grpc.ClientConn, func(), error) {
	pool, err := g.Get(target)
	if err != nil {
		return nil, nil, err
	}
	return pool.Acquire(ctx)
}

// Close closes all pools.
func (g *ConnectionPoolGroup) Close() error {
	var lastErr error
	g.pools.Range(func(key, value interface{}) bool {
		if pool, ok := value.(*ConnectionPool); ok {
			if err := pool.Close(); err != nil {
				lastErr = err
			}
		}
		g.pools.Delete(key)
		return true
	})
	return lastErr
}

// Stats returns statistics for all pools.
func (g *ConnectionPoolGroup) Stats() map[string]PoolStats {
	stats := make(map[string]PoolStats)
	g.pools.Range(func(key, value interface{}) bool {
		if pool, ok := value.(*ConnectionPool); ok {
			stats[key.(string)] = pool.Stats()
		}
		return true
	})
	return stats
}
