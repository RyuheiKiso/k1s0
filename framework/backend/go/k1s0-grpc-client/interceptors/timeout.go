package interceptors

import (
	"context"
	"time"

	"google.golang.org/grpc"
)

// TimeoutInterceptor adds a timeout to gRPC calls.
type TimeoutInterceptor struct {
	timeout time.Duration
}

// NewTimeoutInterceptor creates a new timeout interceptor.
func NewTimeoutInterceptor(timeout time.Duration) *TimeoutInterceptor {
	return &TimeoutInterceptor{timeout: timeout}
}

// Unary returns the unary interceptor.
func (i *TimeoutInterceptor) Unary() grpc.UnaryClientInterceptor {
	return func(
		ctx context.Context,
		method string,
		req, reply interface{},
		cc *grpc.ClientConn,
		invoker grpc.UnaryInvoker,
		opts ...grpc.CallOption,
	) error {
		// If context already has a deadline, don't override
		if _, ok := ctx.Deadline(); ok {
			return invoker(ctx, method, req, reply, cc, opts...)
		}

		// Add timeout
		ctx, cancel := context.WithTimeout(ctx, i.timeout)
		defer cancel()

		return invoker(ctx, method, req, reply, cc, opts...)
	}
}

// Stream returns the stream interceptor.
// Note: Streams typically manage their own timeouts differently.
func (i *TimeoutInterceptor) Stream() grpc.StreamClientInterceptor {
	return func(
		ctx context.Context,
		desc *grpc.StreamDesc,
		cc *grpc.ClientConn,
		method string,
		streamer grpc.Streamer,
		opts ...grpc.CallOption,
	) (grpc.ClientStream, error) {
		// For streams, we don't add a timeout by default
		// as streams are long-lived by nature
		return streamer(ctx, desc, cc, method, opts...)
	}
}

// PerMethodTimeoutInterceptor allows different timeouts for different methods.
type PerMethodTimeoutInterceptor struct {
	defaultTimeout time.Duration
	methodTimeouts map[string]time.Duration
}

// NewPerMethodTimeoutInterceptor creates a new per-method timeout interceptor.
func NewPerMethodTimeoutInterceptor(defaultTimeout time.Duration, methodTimeouts map[string]time.Duration) *PerMethodTimeoutInterceptor {
	return &PerMethodTimeoutInterceptor{
		defaultTimeout: defaultTimeout,
		methodTimeouts: methodTimeouts,
	}
}

// Unary returns the unary interceptor.
func (i *PerMethodTimeoutInterceptor) Unary() grpc.UnaryClientInterceptor {
	return func(
		ctx context.Context,
		method string,
		req, reply interface{},
		cc *grpc.ClientConn,
		invoker grpc.UnaryInvoker,
		opts ...grpc.CallOption,
	) error {
		// If context already has a deadline, don't override
		if _, ok := ctx.Deadline(); ok {
			return invoker(ctx, method, req, reply, cc, opts...)
		}

		// Get timeout for this method
		timeout := i.defaultTimeout
		if t, ok := i.methodTimeouts[method]; ok {
			timeout = t
		}

		// Add timeout
		ctx, cancel := context.WithTimeout(ctx, timeout)
		defer cancel()

		return invoker(ctx, method, req, reply, cc, opts...)
	}
}
