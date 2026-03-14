package buildingblocks

import (
	"context"
	"sync"
)

// BindingInvocation records a single call to InMemoryOutputBinding.Invoke.
type BindingInvocation struct {
	Operation string
	Data      []byte
	Metadata  map[string]string
}

// Compile-time interface compliance check.
var _ OutputBinding = (*InMemoryOutputBinding)(nil)

// InMemoryOutputBinding is a test-oriented OutputBinding that records invocations.
type InMemoryOutputBinding struct {
	mu           sync.Mutex
	last         *BindingInvocation
	mockResponse *BindingResponse
	mockErr      error
	status       ComponentStatus
}

// NewInMemoryOutputBinding creates a new InMemoryOutputBinding.
func NewInMemoryOutputBinding() *InMemoryOutputBinding {
	return &InMemoryOutputBinding{status: StatusUninitialized}
}

func (b *InMemoryOutputBinding) Name() string    { return "inmemory-binding" }
func (b *InMemoryOutputBinding) Version() string { return "1.0.0" }

func (b *InMemoryOutputBinding) Init(_ context.Context, _ Metadata) error {
	b.mu.Lock()
	defer b.mu.Unlock()
	b.status = StatusReady
	return nil
}

func (b *InMemoryOutputBinding) Close(_ context.Context) error {
	b.mu.Lock()
	defer b.mu.Unlock()
	b.status = StatusClosed
	return nil
}

func (b *InMemoryOutputBinding) Status(_ context.Context) ComponentStatus {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.status
}

// Invoke records the call and returns the configured mock response.
func (b *InMemoryOutputBinding) Invoke(_ context.Context, operation string, data []byte, metadata map[string]string) (*BindingResponse, error) {
	b.mu.Lock()
	defer b.mu.Unlock()
	b.last = &BindingInvocation{Operation: operation, Data: data, Metadata: metadata}
	if b.mockErr != nil {
		return nil, b.mockErr
	}
	if b.mockResponse != nil {
		return b.mockResponse, nil
	}
	return &BindingResponse{}, nil
}

// LastInvocation returns the last recorded invocation, or nil if none.
func (b *InMemoryOutputBinding) LastInvocation() *BindingInvocation {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.last
}

// SetResponse configures the mock response returned by Invoke.
func (b *InMemoryOutputBinding) SetResponse(resp *BindingResponse, err error) {
	b.mu.Lock()
	defer b.mu.Unlock()
	b.mockResponse = resp
	b.mockErr = err
}

// Reset clears recorded invocations and mock configuration.
func (b *InMemoryOutputBinding) Reset() {
	b.mu.Lock()
	defer b.mu.Unlock()
	b.last = nil
	b.mockResponse = nil
	b.mockErr = nil
}
