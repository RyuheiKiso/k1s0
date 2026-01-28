package k1s0grpcclient

import (
	"context"
	"path"
	"time"

	k1s0observability "github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-observability"
	"go.uber.org/zap"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
)

// NewLoggingInterceptor creates a unary logging interceptor.
func NewLoggingInterceptor(logger *k1s0observability.Logger) grpc.UnaryClientInterceptor {
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
		logger.Debug(ctx, "gRPC client request",
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
			logger.Error(ctx, "gRPC client call failed", fields...)
		} else {
			fields = append(fields, zap.String("grpc.code", "OK"))
			logger.Debug(ctx, "gRPC client call completed", fields...)
		}

		return err
	}
}

// NewStreamLoggingInterceptor creates a stream logging interceptor.
func NewStreamLoggingInterceptor(logger *k1s0observability.Logger) grpc.StreamClientInterceptor {
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
		logger.Debug(ctx, "gRPC client stream started",
			zap.String("grpc.service", service),
			zap.String("grpc.method", methodName),
			zap.String("grpc.target", cc.Target()),
		)

		// Create the stream
		stream, err := streamer(ctx, desc, cc, method, opts...)

		if err != nil {
			duration := time.Since(start)
			st, _ := status.FromError(err)
			logger.Error(ctx, "gRPC client stream failed",
				zap.String("grpc.service", service),
				zap.String("grpc.method", methodName),
				zap.Duration("grpc.duration", duration),
				zap.String("grpc.code", st.Code().String()),
				zap.Error(err),
			)
		}

		return stream, err
	}
}

// NewTracingInterceptor creates a unary tracing interceptor.
func NewTracingInterceptor() grpc.UnaryClientInterceptor {
	return func(
		ctx context.Context,
		method string,
		req, reply interface{},
		cc *grpc.ClientConn,
		invoker grpc.UnaryInvoker,
		opts ...grpc.CallOption,
	) error {
		ctx = injectTracingMetadata(ctx)
		return invoker(ctx, method, req, reply, cc, opts...)
	}
}

// NewStreamTracingInterceptor creates a stream tracing interceptor.
func NewStreamTracingInterceptor() grpc.StreamClientInterceptor {
	return func(
		ctx context.Context,
		desc *grpc.StreamDesc,
		cc *grpc.ClientConn,
		method string,
		streamer grpc.Streamer,
		opts ...grpc.CallOption,
	) (grpc.ClientStream, error) {
		ctx = injectTracingMetadata(ctx)
		return streamer(ctx, desc, cc, method, opts...)
	}
}

// injectTracingMetadata injects tracing headers into the context.
func injectTracingMetadata(ctx context.Context) context.Context {
	rc := k1s0observability.FromContext(ctx)
	if rc == nil {
		return ctx
	}

	md := metadata.MD{}

	// Get existing metadata
	if existing, ok := metadata.FromOutgoingContext(ctx); ok {
		md = existing.Copy()
	}

	// Inject tracing headers
	if rc.TraceID != "" {
		md.Set("x-trace-id", rc.TraceID)
	}
	if rc.SpanID != "" {
		md.Set("x-span-id", rc.SpanID)
	}
	if rc.RequestID != "" {
		md.Set("x-request-id", rc.RequestID)
	}

	return metadata.NewOutgoingContext(ctx, md)
}

// NewTimeoutInterceptor creates a unary timeout interceptor.
func NewTimeoutInterceptor(timeout time.Duration) grpc.UnaryClientInterceptor {
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
		ctx, cancel := context.WithTimeout(ctx, timeout)
		defer cancel()

		return invoker(ctx, method, req, reply, cc, opts...)
	}
}

// NewRetryInterceptor creates a unary retry interceptor.
func NewRetryInterceptor(config *RetryConfig) grpc.UnaryClientInterceptor {
	retryableCodes := make(map[codes.Code]bool)
	for _, code := range parseRetryableCodes(config.RetryableCodes) {
		retryableCodes[code] = true
	}

	return func(
		ctx context.Context,
		method string,
		req, reply interface{},
		cc *grpc.ClientConn,
		invoker grpc.UnaryInvoker,
		opts ...grpc.CallOption,
	) error {
		var lastErr error

		for attempt := 1; attempt <= config.MaxAttempts; attempt++ {
			// Check context before attempting
			select {
			case <-ctx.Done():
				return ctx.Err()
			default:
			}

			// Invoke the call
			err := invoker(ctx, method, req, reply, cc, opts...)
			if err == nil {
				return nil
			}

			lastErr = err

			// Check if retryable
			st, ok := status.FromError(err)
			if !ok || !retryableCodes[st.Code()] {
				return err
			}

			// Don't retry on last attempt
			if attempt == config.MaxAttempts {
				break
			}

			// Calculate backoff
			backoff := calculateBackoff(attempt, config.InitialBackoff, config.MaxBackoff, config.BackoffMultiplier)

			// Wait for backoff or context cancellation
			select {
			case <-ctx.Done():
				return ctx.Err()
			case <-time.After(backoff):
			}
		}

		return lastErr
	}
}

// parseRetryableCodes converts string codes to gRPC codes.
func parseRetryableCodes(strs []string) []codes.Code {
	codeMap := map[string]codes.Code{
		"OK":                  codes.OK,
		"CANCELLED":           codes.Canceled,
		"UNKNOWN":             codes.Unknown,
		"INVALID_ARGUMENT":    codes.InvalidArgument,
		"DEADLINE_EXCEEDED":   codes.DeadlineExceeded,
		"NOT_FOUND":           codes.NotFound,
		"ALREADY_EXISTS":      codes.AlreadyExists,
		"PERMISSION_DENIED":   codes.PermissionDenied,
		"RESOURCE_EXHAUSTED":  codes.ResourceExhausted,
		"FAILED_PRECONDITION": codes.FailedPrecondition,
		"ABORTED":             codes.Aborted,
		"OUT_OF_RANGE":        codes.OutOfRange,
		"UNIMPLEMENTED":       codes.Unimplemented,
		"INTERNAL":            codes.Internal,
		"UNAVAILABLE":         codes.Unavailable,
		"DATA_LOSS":           codes.DataLoss,
		"UNAUTHENTICATED":     codes.Unauthenticated,
	}

	var result []codes.Code
	for _, s := range strs {
		if code, ok := codeMap[s]; ok {
			result = append(result, code)
		}
	}
	return result
}

// calculateBackoff calculates the backoff duration for an attempt.
func calculateBackoff(attempt int, initial, max time.Duration, multiplier float64) time.Duration {
	backoff := float64(initial)
	for i := 1; i < attempt; i++ {
		backoff *= multiplier
	}

	if time.Duration(backoff) > max {
		return max
	}

	return time.Duration(backoff)
}
