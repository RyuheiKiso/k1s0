package buildingblocks

import "context"

// BindingData represents data from an input binding.
type BindingData struct {
	Data     []byte
	Metadata map[string]string
}

// BindingResponse represents a response from an output binding invocation.
type BindingResponse struct {
	Data     []byte
	Metadata map[string]string
}

// InputBinding reads data from external resources.
type InputBinding interface {
	Component
	Read(ctx context.Context) (*BindingData, error)
}

// OutputBinding invokes operations on external resources.
type OutputBinding interface {
	Component
	Invoke(ctx context.Context, operation string, data []byte, metadata map[string]string) (*BindingResponse, error)
}
