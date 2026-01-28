// Package interceptors provides gRPC interceptors for the k1s0 framework.
package interceptors

import (
	"context"
	"time"

	k1s0obs "github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-observability"
	"go.uber.org/zap"
	"google.golang.org/grpc"
	"google.golang.org/grpc/status"
)

// LoggingInterceptor creates a unary server interceptor that logs requests.
func LoggingInterceptor(logger *k1s0obs.Logger) grpc.UnaryServerInterceptor {
	return func(
		ctx context.Context,
		req interface{},
		info *grpc.UnaryServerInfo,
		handler grpc.UnaryHandler,
	) (interface{}, error) {
		start := time.Now()

		// Get or create request context
		reqCtx := k1s0obs.FromContextOrNew(ctx)
		ctx = reqCtx.ToContext(ctx)

		// Call the handler
		resp, err := handler(ctx, req)

		// Calculate duration
		duration := time.Since(start)

		// Extract status code
		statusCode := "OK"
		if err != nil {
			if st, ok := status.FromError(err); ok {
				statusCode = st.Code().String()
			} else {
				statusCode = "UNKNOWN"
			}
		}

		// Log the request
		fields := []zap.Field{
			zap.String("grpc.method", info.FullMethod),
			zap.String("grpc.status", statusCode),
			zap.Duration("grpc.duration", duration),
		}

		if err != nil {
			logger.Error(ctx, "gRPC request failed", append(fields, zap.Error(err))...)
		} else {
			logger.Info(ctx, "gRPC request completed", fields...)
		}

		return resp, err
	}
}

// StreamLoggingInterceptor creates a stream server interceptor that logs requests.
func StreamLoggingInterceptor(logger *k1s0obs.Logger) grpc.StreamServerInterceptor {
	return func(
		srv interface{},
		ss grpc.ServerStream,
		info *grpc.StreamServerInfo,
		handler grpc.StreamHandler,
	) error {
		start := time.Now()
		ctx := ss.Context()

		// Get or create request context
		reqCtx := k1s0obs.FromContextOrNew(ctx)
		ctx = reqCtx.ToContext(ctx)

		// Wrap the stream with the new context
		wrapped := &wrappedServerStream{
			ServerStream: ss,
			ctx:          ctx,
		}

		// Call the handler
		err := handler(srv, wrapped)

		// Calculate duration
		duration := time.Since(start)

		// Extract status code
		statusCode := "OK"
		if err != nil {
			if st, ok := status.FromError(err); ok {
				statusCode = st.Code().String()
			} else {
				statusCode = "UNKNOWN"
			}
		}

		// Log the request
		fields := []zap.Field{
			zap.String("grpc.method", info.FullMethod),
			zap.String("grpc.status", statusCode),
			zap.Duration("grpc.duration", duration),
			zap.Bool("grpc.stream", true),
		}

		if err != nil {
			logger.Error(ctx, "gRPC stream failed", append(fields, zap.Error(err))...)
		} else {
			logger.Info(ctx, "gRPC stream completed", fields...)
		}

		return err
	}
}

// wrappedServerStream wraps a grpc.ServerStream with a custom context.
type wrappedServerStream struct {
	grpc.ServerStream
	ctx context.Context
}

// Context returns the wrapped context.
func (w *wrappedServerStream) Context() context.Context {
	return w.ctx
}
