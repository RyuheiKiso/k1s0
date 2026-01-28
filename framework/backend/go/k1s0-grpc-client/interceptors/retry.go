package interceptors

import (
	"context"
	"math"
	"math/rand"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// RetryConfig holds retry configuration.
type RetryConfig struct {
	MaxAttempts       int
	InitialBackoff    time.Duration
	MaxBackoff        time.Duration
	BackoffMultiplier float64
	RetryableCodes    []codes.Code
}

// DefaultRetryConfig returns default retry configuration.
func DefaultRetryConfig() *RetryConfig {
	return &RetryConfig{
		MaxAttempts:       3,
		InitialBackoff:    100 * time.Millisecond,
		MaxBackoff:        10 * time.Second,
		BackoffMultiplier: 2.0,
		RetryableCodes: []codes.Code{
			codes.Unavailable,
			codes.ResourceExhausted,
			codes.Aborted,
		},
	}
}

// RetryInterceptor implements retry logic for gRPC calls.
type RetryInterceptor struct {
	config       *RetryConfig
	retryableMap map[codes.Code]bool
}

// NewRetryInterceptor creates a new retry interceptor.
func NewRetryInterceptor(config *RetryConfig) *RetryInterceptor {
	if config == nil {
		config = DefaultRetryConfig()
	}

	retryableMap := make(map[codes.Code]bool)
	for _, code := range config.RetryableCodes {
		retryableMap[code] = true
	}

	return &RetryInterceptor{
		config:       config,
		retryableMap: retryableMap,
	}
}

// Unary returns the unary interceptor.
func (i *RetryInterceptor) Unary() grpc.UnaryClientInterceptor {
	return func(
		ctx context.Context,
		method string,
		req, reply interface{},
		cc *grpc.ClientConn,
		invoker grpc.UnaryInvoker,
		opts ...grpc.CallOption,
	) error {
		var lastErr error

		for attempt := 1; attempt <= i.config.MaxAttempts; attempt++ {
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
			if !ok || !i.isRetryable(st.Code()) {
				return err
			}

			// Don't retry on last attempt
			if attempt == i.config.MaxAttempts {
				break
			}

			// Calculate backoff
			backoff := i.calculateBackoff(attempt)

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

// isRetryable checks if the error code is retryable.
func (i *RetryInterceptor) isRetryable(code codes.Code) bool {
	return i.retryableMap[code]
}

// calculateBackoff calculates the backoff duration for an attempt.
func (i *RetryInterceptor) calculateBackoff(attempt int) time.Duration {
	backoff := float64(i.config.InitialBackoff) * math.Pow(i.config.BackoffMultiplier, float64(attempt-1))

	// Cap at max backoff
	if backoff > float64(i.config.MaxBackoff) {
		backoff = float64(i.config.MaxBackoff)
	}

	// Add jitter (10%)
	jitter := backoff * 0.1 * (rand.Float64()*2 - 1)
	backoff += jitter

	return time.Duration(backoff)
}

// RetryableCodesFromStrings converts string codes to gRPC codes.
func RetryableCodesFromStrings(strs []string) []codes.Code {
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
