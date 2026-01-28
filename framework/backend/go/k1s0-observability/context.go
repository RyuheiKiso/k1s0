package k1s0observability

import (
	"context"
	"crypto/rand"
	"encoding/hex"

	"go.opentelemetry.io/otel/trace"
)

// contextKey is a type for context keys.
type contextKey string

const (
	requestContextKey contextKey = "k1s0_request_context"
)

// RequestContext holds request-scoped observability data.
type RequestContext struct {
	// TraceID is the distributed trace ID.
	TraceID string

	// RequestID is a unique identifier for this request.
	RequestID string

	// SpanID is the current span ID (if tracing is active).
	SpanID string

	// UserID is the authenticated user's ID (if available).
	UserID string

	// TenantID is the tenant ID (if multi-tenant).
	TenantID string

	// Extra holds additional context values.
	Extra map[string]string
}

// NewRequestContext creates a new RequestContext with generated IDs.
func NewRequestContext() *RequestContext {
	return &RequestContext{
		TraceID:   generateID(),
		RequestID: generateID(),
		Extra:     make(map[string]string),
	}
}

// NewRequestContextWithTraceID creates a new RequestContext with a specific trace ID.
func NewRequestContextWithTraceID(traceID string) *RequestContext {
	return &RequestContext{
		TraceID:   traceID,
		RequestID: generateID(),
		Extra:     make(map[string]string),
	}
}

// WithUserID sets the user ID.
func (c *RequestContext) WithUserID(userID string) *RequestContext {
	c.UserID = userID
	return c
}

// WithTenantID sets the tenant ID.
func (c *RequestContext) WithTenantID(tenantID string) *RequestContext {
	c.TenantID = tenantID
	return c
}

// WithSpanID sets the span ID.
func (c *RequestContext) WithSpanID(spanID string) *RequestContext {
	c.SpanID = spanID
	return c
}

// WithExtra sets an extra key-value pair.
func (c *RequestContext) WithExtra(key, value string) *RequestContext {
	if c.Extra == nil {
		c.Extra = make(map[string]string)
	}
	c.Extra[key] = value
	return c
}

// ToContext stores the RequestContext in a context.Context.
func (c *RequestContext) ToContext(ctx context.Context) context.Context {
	return context.WithValue(ctx, requestContextKey, c)
}

// FromContext retrieves a RequestContext from a context.Context.
// Returns nil if not found.
func FromContext(ctx context.Context) *RequestContext {
	if ctx == nil {
		return nil
	}
	rc, _ := ctx.Value(requestContextKey).(*RequestContext)
	return rc
}

// FromContextOrNew retrieves a RequestContext from context or creates a new one.
func FromContextOrNew(ctx context.Context) *RequestContext {
	rc := FromContext(ctx)
	if rc != nil {
		return rc
	}
	return NewRequestContext()
}

// ExtractFromOTelSpan creates a RequestContext from an OpenTelemetry span.
func ExtractFromOTelSpan(ctx context.Context) *RequestContext {
	span := trace.SpanFromContext(ctx)
	if !span.SpanContext().IsValid() {
		return NewRequestContext()
	}

	return &RequestContext{
		TraceID:   span.SpanContext().TraceID().String(),
		SpanID:    span.SpanContext().SpanID().String(),
		RequestID: generateID(),
		Extra:     make(map[string]string),
	}
}

// generateID generates a random 16-character hex string.
func generateID() string {
	bytes := make([]byte, 8)
	_, _ = rand.Read(bytes)
	return hex.EncodeToString(bytes)
}
