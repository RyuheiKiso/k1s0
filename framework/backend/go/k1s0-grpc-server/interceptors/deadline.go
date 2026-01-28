package interceptors

import (
	"context"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// DeadlineInterceptor creates a unary server interceptor that enforces deadlines.
func DeadlineInterceptor(defaultDeadline time.Duration) grpc.UnaryServerInterceptor {
	return func(
		ctx context.Context,
		req interface{},
		info *grpc.UnaryServerInfo,
		handler grpc.UnaryHandler,
	) (interface{}, error) {
		// Check if context already has a deadline
		_, hasDeadline := ctx.Deadline()
		if !hasDeadline && defaultDeadline > 0 {
			var cancel context.CancelFunc
			ctx, cancel = context.WithTimeout(ctx, defaultDeadline)
			defer cancel()
		}

		// Check if deadline is already exceeded
		if err := ctx.Err(); err != nil {
			return nil, status.Errorf(codes.DeadlineExceeded, "deadline exceeded before processing")
		}

		return handler(ctx, req)
	}
}

// StreamDeadlineInterceptor creates a stream server interceptor that enforces deadlines.
func StreamDeadlineInterceptor(defaultDeadline time.Duration) grpc.StreamServerInterceptor {
	return func(
		srv interface{},
		ss grpc.ServerStream,
		info *grpc.StreamServerInfo,
		handler grpc.StreamHandler,
	) error {
		ctx := ss.Context()

		// Check if context already has a deadline
		_, hasDeadline := ctx.Deadline()
		if !hasDeadline && defaultDeadline > 0 {
			var cancel context.CancelFunc
			ctx, cancel = context.WithTimeout(ctx, defaultDeadline)
			defer cancel()

			// Wrap the stream with the new context
			ss = &wrappedServerStream{
				ServerStream: ss,
				ctx:          ctx,
			}
		}

		// Check if deadline is already exceeded
		if err := ctx.Err(); err != nil {
			return status.Errorf(codes.DeadlineExceeded, "deadline exceeded before processing")
		}

		return handler(srv, ss)
	}
}

// DeadlineFromContext returns the remaining time until the deadline.
// Returns 0 if there is no deadline.
func DeadlineFromContext(ctx context.Context) time.Duration {
	deadline, ok := ctx.Deadline()
	if !ok {
		return 0
	}
	return time.Until(deadline)
}

// HasDeadline returns true if the context has a deadline.
func HasDeadline(ctx context.Context) bool {
	_, ok := ctx.Deadline()
	return ok
}
