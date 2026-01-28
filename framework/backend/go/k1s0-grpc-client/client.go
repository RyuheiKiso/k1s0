package k1s0grpcclient

import (
	"context"
	"crypto/tls"
	"crypto/x509"
	"fmt"
	"os"
	"sync"

	k1s0observability "github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-observability"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/keepalive"
)

// Client is the main gRPC client.
type Client struct {
	config *ClientConfig
	logger *k1s0observability.Logger
	conns  sync.Map // map[string]*grpc.ClientConn
}

// NewClient creates a new gRPC client.
//
// Example:
//
//	config := k1s0grpcclient.DefaultClientConfig()
//	client, err := k1s0grpcclient.NewClient(config, logger)
//	if err != nil {
//	    log.Fatal(err)
//	}
//
//	conn, err := client.Dial(ctx, "user-service:50051")
//	defer conn.Close()
func NewClient(config *ClientConfig, logger *k1s0observability.Logger) (*Client, error) {
	config = config.Validate()

	return &Client{
		config: config,
		logger: logger,
	}, nil
}

// NewClientWithDefaults creates a new gRPC client with default configuration.
func NewClientWithDefaults(logger *k1s0observability.Logger) (*Client, error) {
	return NewClient(DefaultClientConfig(), logger)
}

// Dial creates a new connection to the given target.
//
// Example:
//
//	conn, err := client.Dial(ctx, "localhost:50051")
//	if err != nil {
//	    log.Fatal(err)
//	}
//	defer conn.Close()
//
//	userClient := pb.NewUserServiceClient(conn)
func (c *Client) Dial(ctx context.Context, target string) (*grpc.ClientConn, error) {
	opts, err := c.buildDialOptions()
	if err != nil {
		return nil, fmt.Errorf("failed to build dial options: %w", err)
	}

	conn, err := grpc.DialContext(ctx, target, opts...)
	if err != nil {
		return nil, fmt.Errorf("failed to dial %s: %w", target, err)
	}

	// Store connection for cleanup
	c.conns.Store(target, conn)

	return conn, nil
}

// DialCached returns a cached connection or creates a new one.
// Connections are cached by target address.
func (c *Client) DialCached(ctx context.Context, target string) (*grpc.ClientConn, error) {
	if conn, ok := c.conns.Load(target); ok {
		return conn.(*grpc.ClientConn), nil
	}

	return c.Dial(ctx, target)
}

// buildDialOptions builds the gRPC dial options.
func (c *Client) buildDialOptions() ([]grpc.DialOption, error) {
	var opts []grpc.DialOption

	// Transport credentials
	if c.config.TLS {
		creds, err := c.buildTLSCredentials()
		if err != nil {
			return nil, err
		}
		opts = append(opts, grpc.WithTransportCredentials(creds))
	} else {
		opts = append(opts, grpc.WithTransportCredentials(insecure.NewCredentials()))
	}

	// Keep-alive
	opts = append(opts, grpc.WithKeepaliveParams(keepalive.ClientParameters{
		Time:                c.config.KeepAlive.Time,
		Timeout:             c.config.KeepAlive.Timeout,
		PermitWithoutStream: c.config.KeepAlive.PermitWithoutStream,
	}))

	// Message size limits
	opts = append(opts, grpc.WithDefaultCallOptions(
		grpc.MaxCallRecvMsgSize(c.config.MaxRecvMsgSize),
		grpc.MaxCallSendMsgSize(c.config.MaxSendMsgSize),
	))

	// User agent
	if c.config.UserAgent != "" {
		opts = append(opts, grpc.WithUserAgent(c.config.UserAgent))
	}

	// Interceptors
	unaryInterceptors := c.buildUnaryInterceptors()
	streamInterceptors := c.buildStreamInterceptors()

	if len(unaryInterceptors) > 0 {
		opts = append(opts, grpc.WithChainUnaryInterceptor(unaryInterceptors...))
	}
	if len(streamInterceptors) > 0 {
		opts = append(opts, grpc.WithChainStreamInterceptor(streamInterceptors...))
	}

	return opts, nil
}

// buildTLSCredentials builds TLS credentials.
func (c *Client) buildTLSCredentials() (credentials.TransportCredentials, error) {
	if c.config.TLSSkipVerify {
		return credentials.NewTLS(&tls.Config{
			InsecureSkipVerify: true,
		}), nil
	}

	if c.config.TLSCertFile != "" {
		certPool := x509.NewCertPool()
		cert, err := os.ReadFile(c.config.TLSCertFile)
		if err != nil {
			return nil, fmt.Errorf("failed to read TLS cert: %w", err)
		}
		if !certPool.AppendCertsFromPEM(cert) {
			return nil, fmt.Errorf("failed to add TLS cert to pool")
		}
		return credentials.NewClientTLSFromCert(certPool, ""), nil
	}

	// Use system cert pool
	return credentials.NewClientTLSFromCert(nil, ""), nil
}

// buildUnaryInterceptors builds the unary interceptors chain.
func (c *Client) buildUnaryInterceptors() []grpc.UnaryClientInterceptor {
	var interceptors []grpc.UnaryClientInterceptor

	if c.config.Interceptors.Timeout {
		interceptors = append(interceptors, NewTimeoutInterceptor(c.config.DefaultTimeout))
	}

	if c.config.Interceptors.Logging && c.logger != nil {
		interceptors = append(interceptors, NewLoggingInterceptor(c.logger))
	}

	if c.config.Interceptors.Tracing {
		interceptors = append(interceptors, NewTracingInterceptor())
	}

	if c.config.Interceptors.Retry && c.config.Retry.Enabled {
		interceptors = append(interceptors, NewRetryInterceptor(&c.config.Retry))
	}

	return interceptors
}

// buildStreamInterceptors builds the stream interceptors chain.
func (c *Client) buildStreamInterceptors() []grpc.StreamClientInterceptor {
	var interceptors []grpc.StreamClientInterceptor

	if c.config.Interceptors.Logging && c.logger != nil {
		interceptors = append(interceptors, NewStreamLoggingInterceptor(c.logger))
	}

	if c.config.Interceptors.Tracing {
		interceptors = append(interceptors, NewStreamTracingInterceptor())
	}

	return interceptors
}

// Close closes all cached connections.
func (c *Client) Close() error {
	var lastErr error
	c.conns.Range(func(key, value interface{}) bool {
		if conn, ok := value.(*grpc.ClientConn); ok {
			if err := conn.Close(); err != nil {
				lastErr = err
			}
		}
		c.conns.Delete(key)
		return true
	})
	return lastErr
}

// Config returns the client configuration.
func (c *Client) Config() *ClientConfig {
	return c.config
}

// ConnectionState returns the connection state for a target.
type ConnectionState struct {
	// Target is the target address.
	Target string

	// State is the connection state.
	State string

	// Connected indicates if the connection is active.
	Connected bool
}

// GetConnectionStates returns the states of all cached connections.
func (c *Client) GetConnectionStates() []ConnectionState {
	var states []ConnectionState
	c.conns.Range(func(key, value interface{}) bool {
		if conn, ok := value.(*grpc.ClientConn); ok {
			state := conn.GetState()
			states = append(states, ConnectionState{
				Target:    key.(string),
				State:     state.String(),
				Connected: state.String() == "READY",
			})
		}
		return true
	})
	return states
}
