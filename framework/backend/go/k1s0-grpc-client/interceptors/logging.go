package interceptors

import (
	"context"
	"path"
	"time"

	k1s0observability "github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-observability"
	"go.uber.org/zap"
	"google.golang.org/grpc"
	"google.golang.org/grpc/status"
)

// LoggingInterceptor logs gRPC client calls.
type LoggingInterceptor struct {
	logger *k1s0observability.Logger
}

// NewLoggingInterceptor creates a new logging interceptor.
func NewLoggingInterceptor(logger *k1s0observability.Logger) *LoggingInterceptor {
	return &LoggingInterceptor{logger: logger}
}

// Unary returns the unary interceptor.
func (i *LoggingInterceptor) Unary() grpc.UnaryClientInterceptor {
	return func(
		ctx context.Context,
		method string,
		req, reply interface{},
		cc *grpc.ClientConn,
		invoker grpc.UnaryInvoker,
		opts ...grpc.CallOption,
	) error {
		start := time.Now()
		service := path.Dir(method)[1:]
		methodName := path.Base(method)

		// Log request
		i.logger.Debug(ctx, "gRPC client request",
			zap.String("grpc.service", service),
			zap.String("grpc.method", methodName),
			zap.String("grpc.target", cc.Target()),
		)

		// Invoke the call
		err := invoker(ctx, method, req, reply, cc, opts...)

		// Log response
		duration := time.Since(start)
		fields := []zap.Field{
			zap.String("grpc.service", service),
			zap.String("grpc.method", methodName),
			zap.String("grpc.target", cc.Target()),
			zap.Duration("grpc.duration", duration),
		}

		if err != nil {
			st, _ := status.FromError(err)
			fields = append(fields,
				zap.String("grpc.code", st.Code().String()),
				zap.Error(err),
			)
			i.logger.Error(ctx, "gRPC client call failed", fields...)
		} else {
			fields = append(fields, zap.String("grpc.code", "OK"))
			i.logger.Debug(ctx, "gRPC client call completed", fields...)
		}

		return err
	}
}

// Stream returns the stream interceptor.
func (i *LoggingInterceptor) Stream() grpc.StreamClientInterceptor {
	return func(
		ctx context.Context,
		desc *grpc.StreamDesc,
		cc *grpc.ClientConn,
		method string,
		streamer grpc.Streamer,
		opts ...grpc.CallOption,
	) (grpc.ClientStream, error) {
		start := time.Now()
		service := path.Dir(method)[1:]
		methodName := path.Base(method)

		// Log stream start
		i.logger.Debug(ctx, "gRPC client stream started",
			zap.String("grpc.service", service),
			zap.String("grpc.method", methodName),
			zap.String("grpc.target", cc.Target()),
			zap.Bool("grpc.client_stream", desc.ClientStreams),
			zap.Bool("grpc.server_stream", desc.ServerStreams),
		)

		// Create the stream
		stream, err := streamer(ctx, desc, cc, method, opts...)

		if err != nil {
			duration := time.Since(start)
			st, _ := status.FromError(err)
			i.logger.Error(ctx, "gRPC client stream failed",
				zap.String("grpc.service", service),
				zap.String("grpc.method", methodName),
				zap.String("grpc.target", cc.Target()),
				zap.Duration("grpc.duration", duration),
				zap.String("grpc.code", st.Code().String()),
				zap.Error(err),
			)
			return nil, err
		}

		// Wrap the stream for logging
		return &loggingClientStream{
			ClientStream: stream,
			logger:       i.logger,
			ctx:          ctx,
			service:      service,
			method:       methodName,
			start:        start,
		}, nil
	}
}

// loggingClientStream wraps a client stream with logging.
type loggingClientStream struct {
	grpc.ClientStream
	logger  *k1s0observability.Logger
	ctx     context.Context
	service string
	method  string
	start   time.Time
}

// SendMsg logs sent messages.
func (s *loggingClientStream) SendMsg(m interface{}) error {
	err := s.ClientStream.SendMsg(m)
	if err != nil {
		s.logger.Debug(s.ctx, "gRPC client stream send failed",
			zap.String("grpc.service", s.service),
			zap.String("grpc.method", s.method),
			zap.Error(err),
		)
	}
	return err
}

// RecvMsg logs received messages.
func (s *loggingClientStream) RecvMsg(m interface{}) error {
	err := s.ClientStream.RecvMsg(m)
	if err != nil {
		s.logger.Debug(s.ctx, "gRPC client stream recv completed",
			zap.String("grpc.service", s.service),
			zap.String("grpc.method", s.method),
			zap.Duration("grpc.duration", time.Since(s.start)),
		)
	}
	return err
}
