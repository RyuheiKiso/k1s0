// Package k1s0grpcserver provides a gRPC server foundation for the k1s0 framework.
//
// This package provides:
//   - Server configuration and lifecycle management
//   - Standard interceptors for logging, tracing, recovery, and deadline
//   - Integration with k1s0-error and k1s0-observability
//
// # Usage
//
//	config := k1s0grpcserver.NewConfig().
//	    WithPort(50051).
//	    WithServiceName("user-service").
//	    WithReflection(true)
//
//	server, err := k1s0grpcserver.NewServer(config, obsConfig)
//	if err != nil {
//	    log.Fatal(err)
//	}
//
//	pb.RegisterUserServiceServer(server.GRPCServer(), &userService{})
//
//	if err := server.Start(); err != nil {
//	    log.Fatal(err)
//	}
package k1s0grpcserver

import "time"

// Config holds gRPC server configuration.
type Config struct {
	// Port is the port to listen on.
	Port int

	// Host is the host to bind to.
	Host string

	// ServiceName is the name of the service.
	ServiceName string

	// EnableReflection enables gRPC reflection for debugging.
	EnableReflection bool

	// EnableHealthCheck enables the gRPC health check service.
	EnableHealthCheck bool

	// MaxRecvMsgSize is the maximum message size the server can receive.
	MaxRecvMsgSize int

	// MaxSendMsgSize is the maximum message size the server can send.
	MaxSendMsgSize int

	// MaxConcurrentStreams is the maximum number of concurrent streams.
	MaxConcurrentStreams uint32

	// ConnectionTimeout is the connection timeout.
	ConnectionTimeout time.Duration

	// DefaultDeadline is the default deadline for requests without one.
	DefaultDeadline time.Duration

	// EnableRecovery enables panic recovery.
	EnableRecovery bool

	// EnableLogging enables request logging.
	EnableLogging bool

	// EnableTracing enables distributed tracing.
	EnableTracing bool

	// EnableDeadline enables deadline enforcement.
	EnableDeadline bool

	// Stream holds stream backpressure configuration.
	Stream StreamBackpressureConfig
}

// NewConfig creates a new Config with default values.
func NewConfig() *Config {
	return &Config{
		Port:                 50051,
		Host:                 "0.0.0.0",
		MaxRecvMsgSize:       4 * 1024 * 1024,  // 4MB
		MaxSendMsgSize:       4 * 1024 * 1024,  // 4MB
		MaxConcurrentStreams: 1000,
		ConnectionTimeout:    120 * time.Second,
		DefaultDeadline:      30 * time.Second,
		EnableRecovery:       true,
		EnableLogging:        true,
		EnableTracing:        true,
		EnableDeadline:       true,
		EnableReflection:     false,
		EnableHealthCheck:    true,
		Stream:               *DefaultStreamBackpressureConfig(),
	}
}

// WithPort sets the port.
func (c *Config) WithPort(port int) *Config {
	c.Port = port
	return c
}

// WithHost sets the host.
func (c *Config) WithHost(host string) *Config {
	c.Host = host
	return c
}

// WithServiceName sets the service name.
func (c *Config) WithServiceName(name string) *Config {
	c.ServiceName = name
	return c
}

// WithReflection enables/disables gRPC reflection.
func (c *Config) WithReflection(enable bool) *Config {
	c.EnableReflection = enable
	return c
}

// WithHealthCheck enables/disables the health check service.
func (c *Config) WithHealthCheck(enable bool) *Config {
	c.EnableHealthCheck = enable
	return c
}

// WithMaxRecvMsgSize sets the maximum receive message size.
func (c *Config) WithMaxRecvMsgSize(size int) *Config {
	c.MaxRecvMsgSize = size
	return c
}

// WithMaxSendMsgSize sets the maximum send message size.
func (c *Config) WithMaxSendMsgSize(size int) *Config {
	c.MaxSendMsgSize = size
	return c
}

// WithMaxConcurrentStreams sets the maximum concurrent streams.
func (c *Config) WithMaxConcurrentStreams(n uint32) *Config {
	c.MaxConcurrentStreams = n
	return c
}

// WithConnectionTimeout sets the connection timeout.
func (c *Config) WithConnectionTimeout(d time.Duration) *Config {
	c.ConnectionTimeout = d
	return c
}

// WithDefaultDeadline sets the default request deadline.
func (c *Config) WithDefaultDeadline(d time.Duration) *Config {
	c.DefaultDeadline = d
	return c
}

// WithRecovery enables/disables panic recovery.
func (c *Config) WithRecovery(enable bool) *Config {
	c.EnableRecovery = enable
	return c
}

// WithLogging enables/disables request logging.
func (c *Config) WithLogging(enable bool) *Config {
	c.EnableLogging = enable
	return c
}

// WithTracing enables/disables distributed tracing.
func (c *Config) WithTracing(enable bool) *Config {
	c.EnableTracing = enable
	return c
}

// WithDeadline enables/disables deadline enforcement.
func (c *Config) WithDeadline(enable bool) *Config {
	c.EnableDeadline = enable
	return c
}

// Address returns the server address in host:port format.
func (c *Config) Address() string {
	return c.Host + ":" + itoa(c.Port)
}

// itoa converts an int to a string.
func itoa(i int) string {
	if i == 0 {
		return "0"
	}
	var b [20]byte
	n := len(b)
	neg := i < 0
	if neg {
		i = -i
	}
	for i > 0 {
		n--
		b[n] = byte('0' + i%10)
		i /= 10
	}
	if neg {
		n--
		b[n] = '-'
	}
	return string(b[n:])
}
