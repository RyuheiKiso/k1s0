package interceptors

import (
	"context"

	k1s0obs "github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-observability"
	"google.golang.org/grpc"
	"google.golang.org/grpc/metadata"
)

// Common metadata keys for tracing.
const (
	TraceIDKey   = "x-trace-id"
	RequestIDKey = "x-request-id"
	SpanIDKey    = "x-span-id"
)

// TracingInterceptor creates a unary server interceptor that handles tracing.
func TracingInterceptor() grpc.UnaryServerInterceptor {
	return func(
		ctx context.Context,
		req interface{},
		info *grpc.UnaryServerInfo,
		handler grpc.UnaryHandler,
	) (interface{}, error) {
		// Extract trace information from incoming metadata
		reqCtx := extractRequestContext(ctx)
		ctx = reqCtx.ToContext(ctx)

		// Call the handler
		resp, err := handler(ctx, req)

		// Add trace information to outgoing metadata
		grpc.SetTrailer(ctx, metadata.Pairs(
			TraceIDKey, reqCtx.TraceID,
			RequestIDKey, reqCtx.RequestID,
		))

		return resp, err
	}
}

// StreamTracingInterceptor creates a stream server interceptor that handles tracing.
func StreamTracingInterceptor() grpc.StreamServerInterceptor {
	return func(
		srv interface{},
		ss grpc.ServerStream,
		info *grpc.StreamServerInfo,
		handler grpc.StreamHandler,
	) error {
		ctx := ss.Context()

		// Extract trace information from incoming metadata
		reqCtx := extractRequestContext(ctx)
		ctx = reqCtx.ToContext(ctx)

		// Wrap the stream with the new context
		wrapped := &wrappedServerStream{
			ServerStream: ss,
			ctx:          ctx,
		}

		// Call the handler
		err := handler(srv, wrapped)

		// Add trace information to outgoing metadata
		ss.SetTrailer(metadata.Pairs(
			TraceIDKey, reqCtx.TraceID,
			RequestIDKey, reqCtx.RequestID,
		))

		return err
	}
}

// extractRequestContext extracts or creates a RequestContext from gRPC metadata.
func extractRequestContext(ctx context.Context) *k1s0obs.RequestContext {
	var traceID, requestID, spanID string

	md, ok := metadata.FromIncomingContext(ctx)
	if ok {
		if values := md.Get(TraceIDKey); len(values) > 0 {
			traceID = values[0]
		}
		if values := md.Get(RequestIDKey); len(values) > 0 {
			requestID = values[0]
		}
		if values := md.Get(SpanIDKey); len(values) > 0 {
			spanID = values[0]
		}
	}

	var reqCtx *k1s0obs.RequestContext
	if traceID != "" {
		reqCtx = k1s0obs.NewRequestContextWithTraceID(traceID)
	} else {
		reqCtx = k1s0obs.NewRequestContext()
	}

	if requestID != "" {
		// Override the generated request ID with the incoming one
		reqCtx = k1s0obs.NewRequestContextWithTraceID(reqCtx.TraceID)
	}

	if spanID != "" {
		reqCtx.WithSpanID(spanID)
	}

	return reqCtx
}

// InjectTraceMetadata injects trace context into outgoing metadata.
func InjectTraceMetadata(ctx context.Context) context.Context {
	reqCtx := k1s0obs.FromContext(ctx)
	if reqCtx == nil {
		return ctx
	}

	md, ok := metadata.FromOutgoingContext(ctx)
	if !ok {
		md = metadata.New(nil)
	}

	md.Set(TraceIDKey, reqCtx.TraceID)
	md.Set(RequestIDKey, reqCtx.RequestID)
	if reqCtx.SpanID != "" {
		md.Set(SpanIDKey, reqCtx.SpanID)
	}

	return metadata.NewOutgoingContext(ctx, md)
}
