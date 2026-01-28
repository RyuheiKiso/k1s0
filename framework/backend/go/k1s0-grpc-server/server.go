package k1s0grpcserver

import (
	"context"
	"fmt"
	"net"

	k1s0obs "github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-observability"
	"github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-grpc-server/interceptors"
	"go.uber.org/zap"
	"google.golang.org/grpc"
	"google.golang.org/grpc/health"
	healthpb "google.golang.org/grpc/health/grpc_health_v1"
	"google.golang.org/grpc/reflection"
)

// Server wraps a gRPC server with k1s0 conventions.
type Server struct {
	config       *Config
	grpcServer   *grpc.Server
	logger       *k1s0obs.Logger
	healthServer *health.Server
	listener     net.Listener
}

// NewServer creates a new Server with the given configuration.
func NewServer(config *Config, obsConfig *k1s0obs.Config) (*Server, error) {
	// Create logger
	logger, err := k1s0obs.NewLogger(obsConfig)
	if err != nil {
		return nil, fmt.Errorf("failed to create logger: %w", err)
	}

	// Build interceptors
	unaryInterceptors := buildUnaryInterceptors(config, logger)
	streamInterceptors := buildStreamInterceptors(config, logger)

	// Create gRPC server options
	opts := []grpc.ServerOption{
		grpc.MaxRecvMsgSize(config.MaxRecvMsgSize),
		grpc.MaxSendMsgSize(config.MaxSendMsgSize),
		grpc.MaxConcurrentStreams(config.MaxConcurrentStreams),
		grpc.ChainUnaryInterceptor(unaryInterceptors...),
		grpc.ChainStreamInterceptor(streamInterceptors...),
	}

	// Create gRPC server
	grpcServer := grpc.NewServer(opts...)

	// Create health server if enabled
	var healthServer *health.Server
	if config.EnableHealthCheck {
		healthServer = health.NewServer()
		healthpb.RegisterHealthServer(grpcServer, healthServer)
	}

	// Enable reflection if configured
	if config.EnableReflection {
		reflection.Register(grpcServer)
	}

	return &Server{
		config:       config,
		grpcServer:   grpcServer,
		logger:       logger,
		healthServer: healthServer,
	}, nil
}

// buildUnaryInterceptors builds the chain of unary interceptors.
func buildUnaryInterceptors(config *Config, logger *k1s0obs.Logger) []grpc.UnaryServerInterceptor {
	var chain []grpc.UnaryServerInterceptor

	// Recovery should be first to catch panics from other interceptors
	if config.EnableRecovery {
		chain = append(chain, interceptors.RecoveryInterceptor(logger, nil))
	}

	// Tracing should be early to establish context
	if config.EnableTracing {
		chain = append(chain, interceptors.TracingInterceptor())
	}

	// Deadline enforcement
	if config.EnableDeadline {
		chain = append(chain, interceptors.DeadlineInterceptor(config.DefaultDeadline))
	}

	// Logging
	if config.EnableLogging {
		chain = append(chain, interceptors.LoggingInterceptor(logger))
	}

	// Error conversion should be last to convert errors from handlers
	chain = append(chain, interceptors.ErrorInterceptor())

	return chain
}

// buildStreamInterceptors builds the chain of stream interceptors.
func buildStreamInterceptors(config *Config, logger *k1s0obs.Logger) []grpc.StreamServerInterceptor {
	var chain []grpc.StreamServerInterceptor

	// Recovery should be first
	if config.EnableRecovery {
		chain = append(chain, interceptors.StreamRecoveryInterceptor(logger, nil))
	}

	// Tracing
	if config.EnableTracing {
		chain = append(chain, interceptors.StreamTracingInterceptor())
	}

	// Deadline enforcement
	if config.EnableDeadline {
		chain = append(chain, interceptors.StreamDeadlineInterceptor(config.DefaultDeadline))
	}

	// Logging
	if config.EnableLogging {
		chain = append(chain, interceptors.StreamLoggingInterceptor(logger))
	}

	// Error conversion
	chain = append(chain, interceptors.StreamErrorInterceptor())

	return chain
}

// GRPCServer returns the underlying gRPC server.
func (s *Server) GRPCServer() *grpc.Server {
	return s.grpcServer
}

// HealthServer returns the health server.
func (s *Server) HealthServer() *health.Server {
	return s.healthServer
}

// Logger returns the logger.
func (s *Server) Logger() *k1s0obs.Logger {
	return s.logger
}

// Config returns the server configuration.
func (s *Server) Config() *Config {
	return s.config
}

// Start starts the gRPC server.
func (s *Server) Start() error {
	addr := s.config.Address()

	listener, err := net.Listen("tcp", addr)
	if err != nil {
		return fmt.Errorf("failed to listen on %s: %w", addr, err)
	}
	s.listener = listener

	s.logger.Info(context.Background(), "gRPC server starting",
		zap.String("address", addr),
		zap.String("service", s.config.ServiceName),
	)

	// Set health status to serving
	if s.healthServer != nil {
		s.healthServer.SetServingStatus("", healthpb.HealthCheckResponse_SERVING)
	}

	return s.grpcServer.Serve(listener)
}

// Stop gracefully stops the gRPC server.
func (s *Server) Stop() {
	s.logger.Info(context.Background(), "gRPC server stopping")

	// Set health status to not serving
	if s.healthServer != nil {
		s.healthServer.SetServingStatus("", healthpb.HealthCheckResponse_NOT_SERVING)
	}

	s.grpcServer.GracefulStop()
}

// ForceStop immediately stops the gRPC server.
func (s *Server) ForceStop() {
	s.logger.Info(context.Background(), "gRPC server force stopping")
	s.grpcServer.Stop()
}

// SetServiceHealth sets the health status of a service.
func (s *Server) SetServiceHealth(service string, serving bool) {
	if s.healthServer == nil {
		return
	}

	status := healthpb.HealthCheckResponse_SERVING
	if !serving {
		status = healthpb.HealthCheckResponse_NOT_SERVING
	}
	s.healthServer.SetServingStatus(service, status)
}

// Address returns the server's listening address.
// Returns empty string if the server hasn't started.
func (s *Server) Address() string {
	if s.listener == nil {
		return ""
	}
	return s.listener.Addr().String()
}
