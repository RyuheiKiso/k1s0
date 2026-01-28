package interceptors

import (
	"context"
	"errors"

	k1s0error "github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-error"
	"github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-error/presentation"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
)

// ErrorInterceptor creates a unary server interceptor that converts k1s0 errors to gRPC errors.
func ErrorInterceptor() grpc.UnaryServerInterceptor {
	return func(
		ctx context.Context,
		req interface{},
		info *grpc.UnaryServerInfo,
		handler grpc.UnaryHandler,
	) (interface{}, error) {
		resp, err := handler(ctx, req)
		if err != nil {
			return resp, convertError(ctx, err)
		}
		return resp, nil
	}
}

// StreamErrorInterceptor creates a stream server interceptor that converts k1s0 errors to gRPC errors.
func StreamErrorInterceptor() grpc.StreamServerInterceptor {
	return func(
		srv interface{},
		ss grpc.ServerStream,
		info *grpc.StreamServerInfo,
		handler grpc.StreamHandler,
	) error {
		err := handler(srv, ss)
		if err != nil {
			return convertError(ss.Context(), err)
		}
		return nil
	}
}

// convertError converts a k1s0 error to a gRPC error.
func convertError(ctx context.Context, err error) error {
	// Check if it's already a gRPC error
	if _, ok := status.FromError(err); ok {
		return err
	}

	// Check for AppError
	var appErr *k1s0error.AppError
	if errors.As(err, &appErr) {
		grpcErr := appErr.ToGRPCError()
		return createGRPCError(ctx, grpcErr)
	}

	// Check for DomainError
	var domainErr *k1s0error.DomainError
	if errors.As(err, &domainErr) {
		grpcErr := domainErr.ToGRPCError()
		return createGRPCError(ctx, grpcErr)
	}

	// Unknown error - wrap as internal
	return status.Errorf(codes.Internal, "internal error: %v", err)
}

// createGRPCError creates a gRPC error with metadata from a GRPCError.
func createGRPCError(ctx context.Context, grpcErr *presentation.GRPCError) error {
	// Set trailer metadata with error details
	md := metadata.Pairs()
	for k, v := range grpcErr.Metadata() {
		md.Append(k, v)
	}
	grpc.SetTrailer(ctx, md)

	// Convert to gRPC status code
	code := grpcStatusCode(grpcErr.StatusCode())

	return status.Error(code, grpcErr.Message())
}

// grpcStatusCode converts a presentation.GRPCStatusCode to codes.Code.
func grpcStatusCode(c presentation.GRPCStatusCode) codes.Code {
	switch c {
	case presentation.GRPCCodeOK:
		return codes.OK
	case presentation.GRPCCodeCancelled:
		return codes.Canceled
	case presentation.GRPCCodeUnknown:
		return codes.Unknown
	case presentation.GRPCCodeInvalidArgument:
		return codes.InvalidArgument
	case presentation.GRPCCodeDeadlineExceeded:
		return codes.DeadlineExceeded
	case presentation.GRPCCodeNotFound:
		return codes.NotFound
	case presentation.GRPCCodeAlreadyExists:
		return codes.AlreadyExists
	case presentation.GRPCCodePermissionDenied:
		return codes.PermissionDenied
	case presentation.GRPCCodeResourceExhausted:
		return codes.ResourceExhausted
	case presentation.GRPCCodeFailedPrecondition:
		return codes.FailedPrecondition
	case presentation.GRPCCodeAborted:
		return codes.Aborted
	case presentation.GRPCCodeOutOfRange:
		return codes.OutOfRange
	case presentation.GRPCCodeUnimplemented:
		return codes.Unimplemented
	case presentation.GRPCCodeInternal:
		return codes.Internal
	case presentation.GRPCCodeUnavailable:
		return codes.Unavailable
	case presentation.GRPCCodeDataLoss:
		return codes.DataLoss
	case presentation.GRPCCodeUnauthenticated:
		return codes.Unauthenticated
	default:
		return codes.Internal
	}
}
