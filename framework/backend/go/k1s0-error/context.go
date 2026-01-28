package k1s0error

// ErrorContext holds correlation information for errors.
type ErrorContext struct {
	TraceID   string
	RequestID string
	TenantID  string
	UserID    string
	Extra     map[string]string
}

// NewErrorContext creates a new empty ErrorContext.
func NewErrorContext() *ErrorContext {
	return &ErrorContext{
		Extra: make(map[string]string),
	}
}

// WithTraceID sets the trace ID and returns the context.
func (c *ErrorContext) WithTraceID(traceID string) *ErrorContext {
	c.TraceID = traceID
	return c
}

// WithRequestID sets the request ID and returns the context.
func (c *ErrorContext) WithRequestID(requestID string) *ErrorContext {
	c.RequestID = requestID
	return c
}

// WithTenantID sets the tenant ID and returns the context.
func (c *ErrorContext) WithTenantID(tenantID string) *ErrorContext {
	c.TenantID = tenantID
	return c
}

// WithUserID sets the user ID and returns the context.
func (c *ErrorContext) WithUserID(userID string) *ErrorContext {
	c.UserID = userID
	return c
}

// WithExtra adds an extra key-value pair and returns the context.
func (c *ErrorContext) WithExtra(key, value string) *ErrorContext {
	if c.Extra == nil {
		c.Extra = make(map[string]string)
	}
	c.Extra[key] = value
	return c
}

// Merge merges another context into this one.
// Values from the other context override values in this context.
func (c *ErrorContext) Merge(other *ErrorContext) *ErrorContext {
	if other == nil {
		return c
	}
	if other.TraceID != "" {
		c.TraceID = other.TraceID
	}
	if other.RequestID != "" {
		c.RequestID = other.RequestID
	}
	if other.TenantID != "" {
		c.TenantID = other.TenantID
	}
	if other.UserID != "" {
		c.UserID = other.UserID
	}
	for k, v := range other.Extra {
		c.WithExtra(k, v)
	}
	return c
}

// Clone creates a copy of the context.
func (c *ErrorContext) Clone() *ErrorContext {
	clone := &ErrorContext{
		TraceID:   c.TraceID,
		RequestID: c.RequestID,
		TenantID:  c.TenantID,
		UserID:    c.UserID,
		Extra:     make(map[string]string),
	}
	for k, v := range c.Extra {
		clone.Extra[k] = v
	}
	return clone
}
