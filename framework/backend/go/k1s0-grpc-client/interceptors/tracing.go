package interceptors

import (
	"context"
	"path"

	k1s0observability "github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-observability"
	"google.golang.org/grpc"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
)

// TracingInterceptor adds tracing context to gRPC calls.
type TracingInterceptor struct{}

// NewTracingInterceptor creates a new tracing interceptor.
func NewTracingInterceptor() *TracingInterceptor {
	return &TracingInterceptor{}
}

// Unary returns the unary interceptor.
func (i *TracingInterceptor) Unary() grpc.UnaryClientInterceptor {
	return func(
		ctx context.Context,
		method string,
		req, reply interface{},
		cc *grpc.ClientConn,
		invoker grpc.UnaryInvoker,
		opts ...grpc.CallOption,
	) error {
		// Extract tracing context
		ctx = i.injectTracing(ctx)

		// Create span name
		_ = path.Base(method)

		// Invoke the call
		err := invoker(ctx, method, req, reply, cc, opts...)

		// Record status
		if err != nil {
			st, _ := status.FromError(err)
			_ = st.Code().String()
		}

		return err
	}
}

// Stream returns the stream interceptor.
func (i *TracingInterceptor) Stream() grpc.StreamClientInterceptor {
	return func(
		ctx context.Context,
		desc *grpc.StreamDesc,
		cc *grpc.ClientConn,
		method string,
		streamer grpc.Streamer,
		opts ...grpc.CallOption,
	) (grpc.ClientStream, error) {
		// Extract tracing context
		ctx = i.injectTracing(ctx)

		// Create the stream
		stream, err := streamer(ctx, desc, cc, method, opts...)
		if err != nil {
			return nil, err
		}

		return &tracingClientStream{
			ClientStream: stream,
		}, nil
	}
}

// injectTracing injects tracing headers into the context.
func (i *TracingInterceptor) injectTracing(ctx context.Context) context.Context {
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

// tracingClientStream wraps a client stream with tracing.
type tracingClientStream struct {
	grpc.ClientStream
}

// MetadataInterceptor adds custom metadata to gRPC calls.
type MetadataInterceptor struct {
	metadata map[string]string
}

// NewMetadataInterceptor creates a new metadata interceptor.
func NewMetadataInterceptor(md map[string]string) *MetadataInterceptor {
	return &MetadataInterceptor{metadata: md}
}

// Unary returns the unary interceptor.
func (i *MetadataInterceptor) Unary() grpc.UnaryClientInterceptor {
	return func(
		ctx context.Context,
		method string,
		req, reply interface{},
		cc *grpc.ClientConn,
		invoker grpc.UnaryInvoker,
		opts ...grpc.CallOption,
	) error {
		ctx = i.injectMetadata(ctx)
		return invoker(ctx, method, req, reply, cc, opts...)
	}
}

// Stream returns the stream interceptor.
func (i *MetadataInterceptor) Stream() grpc.StreamClientInterceptor {
	return func(
		ctx context.Context,
		desc *grpc.StreamDesc,
		cc *grpc.ClientConn,
		method string,
		streamer grpc.Streamer,
		opts ...grpc.CallOption,
	) (grpc.ClientStream, error) {
		ctx = i.injectMetadata(ctx)
		return streamer(ctx, desc, cc, method, opts...)
	}
}

// injectMetadata injects custom metadata into the context.
func (i *MetadataInterceptor) injectMetadata(ctx context.Context) context.Context {
	md := metadata.MD{}

	// Get existing metadata
	if existing, ok := metadata.FromOutgoingContext(ctx); ok {
		md = existing.Copy()
	}

	// Inject custom metadata
	for k, v := range i.metadata {
		md.Set(k, v)
	}

	return metadata.NewOutgoingContext(ctx, md)
}
