// Package k1s0grpcclient provides gRPC client utilities for the k1s0 framework.
//
// This package implements:
//   - Connection management with pooling
//   - Client-side interceptors for logging, tracing, retry, and timeout
//   - Integration with k1s0-resilience for circuit breaker patterns
//
// # Configuration
//
// Example config.yaml:
//
//	grpc_client:
//	  default_timeout: 30s
//	  keep_alive:
//	    time: 10s
//	    timeout: 3s
//	    permit_without_stream: true
//	  retry:
//	    max_attempts: 3
//	    initial_backoff: 100ms
//	    max_backoff: 10s
//	  targets:
//	    user-service:
//	      address: user-service:50051
//	      timeout: 5s
//	      tls: true
//
// # Usage
//
//	client, err := k1s0grpcclient.NewClient(clientConfig)
//
//	conn, err := client.Dial(ctx, "user-service:50051")
//	defer conn.Close()
//
//	userClient := pb.NewUserServiceClient(conn)
//	resp, err := userClient.GetUser(ctx, &pb.GetUserRequest{Id: "123"})
package k1s0grpcclient

import (
	"time"
)

// ClientConfig holds gRPC client configuration.
type ClientConfig struct {
	// DefaultTimeout is the default timeout for RPC calls.
	// Default is 30 seconds.
	DefaultTimeout time.Duration

	// KeepAlive is the keep-alive configuration.
	KeepAlive KeepAliveConfig

	// Retry is the retry configuration.
	Retry RetryConfig

	// Interceptors configures which interceptors to enable.
	Interceptors InterceptorConfig

	// TLS enables TLS for connections.
	TLS bool

	// TLSSkipVerify skips TLS certificate verification (not recommended for production).
	TLSSkipVerify bool

	// TLSCertFile is the path to the TLS certificate file.
	TLSCertFile string

	// MaxRecvMsgSize is the maximum message size the client can receive.
	// Default is 4MB.
	MaxRecvMsgSize int

	// MaxSendMsgSize is the maximum message size the client can send.
	// Default is 4MB.
	MaxSendMsgSize int

	// UserAgent is the user agent to send with requests.
	UserAgent string
}

// KeepAliveConfig holds keep-alive configuration.
type KeepAliveConfig struct {
	// Time is the interval between keep-alive pings.
	// Default is 10 seconds.
	Time time.Duration

	// Timeout is the timeout for keep-alive pings.
	// Default is 3 seconds.
	Timeout time.Duration

	// PermitWithoutStream allows keep-alive pings even when there are no active streams.
	// Default is true.
	PermitWithoutStream bool
}

// RetryConfig holds retry configuration.
type RetryConfig struct {
	// Enabled enables automatic retry.
	// Default is true.
	Enabled bool

	// MaxAttempts is the maximum number of retry attempts.
	// Default is 3.
	MaxAttempts int

	// InitialBackoff is the initial backoff duration.
	// Default is 100ms.
	InitialBackoff time.Duration

	// MaxBackoff is the maximum backoff duration.
	// Default is 10 seconds.
	MaxBackoff time.Duration

	// BackoffMultiplier is the backoff multiplier.
	// Default is 2.0.
	BackoffMultiplier float64

	// RetryableCodes is the list of gRPC status codes that should be retried.
	// Default is [UNAVAILABLE, RESOURCE_EXHAUSTED].
	RetryableCodes []string
}

// InterceptorConfig configures which interceptors to enable.
type InterceptorConfig struct {
	// Logging enables logging interceptor.
	// Default is true.
	Logging bool

	// Tracing enables tracing interceptor.
	// Default is true.
	Tracing bool

	// Retry enables retry interceptor.
	// Default is true.
	Retry bool

	// Timeout enables timeout interceptor.
	// Default is true.
	Timeout bool

	// CircuitBreaker enables circuit breaker interceptor.
	// Default is false.
	CircuitBreaker bool
}

// DefaultClientConfig returns a ClientConfig with default values.
func DefaultClientConfig() *ClientConfig {
	return &ClientConfig{
		DefaultTimeout: 30 * time.Second,
		KeepAlive: KeepAliveConfig{
			Time:                10 * time.Second,
			Timeout:             3 * time.Second,
			PermitWithoutStream: true,
		},
		Retry: RetryConfig{
			Enabled:           true,
			MaxAttempts:       3,
			InitialBackoff:    100 * time.Millisecond,
			MaxBackoff:        10 * time.Second,
			BackoffMultiplier: 2.0,
			RetryableCodes:    []string{"UNAVAILABLE", "RESOURCE_EXHAUSTED"},
		},
		Interceptors: InterceptorConfig{
			Logging:        true,
			Tracing:        true,
			Retry:          true,
			Timeout:        true,
			CircuitBreaker: false,
		},
		MaxRecvMsgSize: 4 * 1024 * 1024, // 4MB
		MaxSendMsgSize: 4 * 1024 * 1024, // 4MB
	}
}

// Validate validates the client configuration and applies defaults.
func (c *ClientConfig) Validate() *ClientConfig {
	if c.DefaultTimeout <= 0 {
		c.DefaultTimeout = 30 * time.Second
	}
	if c.KeepAlive.Time <= 0 {
		c.KeepAlive.Time = 10 * time.Second
	}
	if c.KeepAlive.Timeout <= 0 {
		c.KeepAlive.Timeout = 3 * time.Second
	}
	if c.Retry.MaxAttempts <= 0 {
		c.Retry.MaxAttempts = 3
	}
	if c.Retry.InitialBackoff <= 0 {
		c.Retry.InitialBackoff = 100 * time.Millisecond
	}
	if c.Retry.MaxBackoff <= 0 {
		c.Retry.MaxBackoff = 10 * time.Second
	}
	if c.Retry.BackoffMultiplier <= 0 {
		c.Retry.BackoffMultiplier = 2.0
	}
	if len(c.Retry.RetryableCodes) == 0 {
		c.Retry.RetryableCodes = []string{"UNAVAILABLE", "RESOURCE_EXHAUSTED"}
	}
	if c.MaxRecvMsgSize <= 0 {
		c.MaxRecvMsgSize = 4 * 1024 * 1024
	}
	if c.MaxSendMsgSize <= 0 {
		c.MaxSendMsgSize = 4 * 1024 * 1024
	}
	return c
}

// ClientConfigBuilder builds a ClientConfig.
type ClientConfigBuilder struct {
	config *ClientConfig
}

// NewClientConfigBuilder creates a new ClientConfigBuilder.
func NewClientConfigBuilder() *ClientConfigBuilder {
	return &ClientConfigBuilder{
		config: DefaultClientConfig(),
	}
}

// DefaultTimeout sets the default timeout.
func (b *ClientConfigBuilder) DefaultTimeout(timeout time.Duration) *ClientConfigBuilder {
	b.config.DefaultTimeout = timeout
	return b
}

// KeepAliveTime sets the keep-alive time.
func (b *ClientConfigBuilder) KeepAliveTime(t time.Duration) *ClientConfigBuilder {
	b.config.KeepAlive.Time = t
	return b
}

// KeepAliveTimeout sets the keep-alive timeout.
func (b *ClientConfigBuilder) KeepAliveTimeout(t time.Duration) *ClientConfigBuilder {
	b.config.KeepAlive.Timeout = t
	return b
}

// MaxRetryAttempts sets the maximum retry attempts.
func (b *ClientConfigBuilder) MaxRetryAttempts(attempts int) *ClientConfigBuilder {
	b.config.Retry.MaxAttempts = attempts
	return b
}

// RetryBackoff sets the retry backoff configuration.
func (b *ClientConfigBuilder) RetryBackoff(initial, max time.Duration, multiplier float64) *ClientConfigBuilder {
	b.config.Retry.InitialBackoff = initial
	b.config.Retry.MaxBackoff = max
	b.config.Retry.BackoffMultiplier = multiplier
	return b
}

// EnableLogging enables or disables the logging interceptor.
func (b *ClientConfigBuilder) EnableLogging(enabled bool) *ClientConfigBuilder {
	b.config.Interceptors.Logging = enabled
	return b
}

// EnableTracing enables or disables the tracing interceptor.
func (b *ClientConfigBuilder) EnableTracing(enabled bool) *ClientConfigBuilder {
	b.config.Interceptors.Tracing = enabled
	return b
}

// EnableRetry enables or disables the retry interceptor.
func (b *ClientConfigBuilder) EnableRetry(enabled bool) *ClientConfigBuilder {
	b.config.Interceptors.Retry = enabled
	b.config.Retry.Enabled = enabled
	return b
}

// EnableCircuitBreaker enables or disables the circuit breaker interceptor.
func (b *ClientConfigBuilder) EnableCircuitBreaker(enabled bool) *ClientConfigBuilder {
	b.config.Interceptors.CircuitBreaker = enabled
	return b
}

// TLS enables TLS with the given certificate file.
func (b *ClientConfigBuilder) TLS(certFile string) *ClientConfigBuilder {
	b.config.TLS = true
	b.config.TLSCertFile = certFile
	return b
}

// Insecure disables TLS.
func (b *ClientConfigBuilder) Insecure() *ClientConfigBuilder {
	b.config.TLS = false
	return b
}

// UserAgent sets the user agent.
func (b *ClientConfigBuilder) UserAgent(userAgent string) *ClientConfigBuilder {
	b.config.UserAgent = userAgent
	return b
}

// MaxMessageSize sets the maximum message size for both send and receive.
func (b *ClientConfigBuilder) MaxMessageSize(size int) *ClientConfigBuilder {
	b.config.MaxRecvMsgSize = size
	b.config.MaxSendMsgSize = size
	return b
}

// Build creates the ClientConfig.
func (b *ClientConfigBuilder) Build() *ClientConfig {
	return b.config.Validate()
}
