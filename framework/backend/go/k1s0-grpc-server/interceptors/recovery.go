package interceptors

import (
	"context"
	"runtime/debug"

	k1s0obs "github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-observability"
	"go.uber.org/zap"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// PanicHandler is called when a panic is recovered.
type PanicHandler func(ctx context.Context, p interface{}, stack []byte)

// RecoveryInterceptor creates a unary server interceptor that recovers from panics.
func RecoveryInterceptor(logger *k1s0obs.Logger, handler PanicHandler) grpc.UnaryServerInterceptor {
	return func(
		ctx context.Context,
		req interface{},
		info *grpc.UnaryServerInfo,
		h grpc.UnaryHandler,
	) (resp interface{}, err error) {
		defer func() {
			if p := recover(); p != nil {
				stack := debug.Stack()

				// Log the panic
				logger.Error(ctx, "gRPC handler panic recovered",
					zap.Any("panic", p),
					zap.String("grpc.method", info.FullMethod),
					zap.ByteString("stack", stack),
				)

				// Call custom handler if provided
				if handler != nil {
					handler(ctx, p, stack)
				}

				// Return an internal error
				err = status.Errorf(codes.Internal, "internal error")
			}
		}()

		return h(ctx, req)
	}
}

// StreamRecoveryInterceptor creates a stream server interceptor that recovers from panics.
func StreamRecoveryInterceptor(logger *k1s0obs.Logger, handler PanicHandler) grpc.StreamServerInterceptor {
	return func(
		srv interface{},
		ss grpc.ServerStream,
		info *grpc.StreamServerInfo,
		h grpc.StreamHandler,
	) (err error) {
		ctx := ss.Context()

		defer func() {
			if p := recover(); p != nil {
				stack := debug.Stack()

				// Log the panic
				logger.Error(ctx, "gRPC stream handler panic recovered",
					zap.Any("panic", p),
					zap.String("grpc.method", info.FullMethod),
					zap.ByteString("stack", stack),
				)

				// Call custom handler if provided
				if handler != nil {
					handler(ctx, p, stack)
				}

				// Return an internal error
				err = status.Errorf(codes.Internal, "internal error")
			}
		}()

		return h(srv, ss)
	}
}

// DefaultPanicHandler is a simple panic handler that does nothing.
// Use this as a placeholder or override with your own implementation.
var DefaultPanicHandler PanicHandler = nil
