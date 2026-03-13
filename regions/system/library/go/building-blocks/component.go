package buildingblocks

import (
	"context"
)

// ComponentStatus represents the current state of a component.
type ComponentStatus string

const (
	StatusUninitialized ComponentStatus = "uninitialized"
	StatusReady         ComponentStatus = "ready"
	StatusDegraded      ComponentStatus = "degraded"
	StatusClosed        ComponentStatus = "closed"
	StatusError         ComponentStatus = "error"
)

// Metadata holds component metadata.
type Metadata struct {
	Name    string            `json:"name"`
	Version string            `json:"version"`
	Tags    map[string]string `json:"tags,omitempty"`
}

// Component is the base interface for all building block components.
type Component interface {
	Name() string
	Version() string
	Init(ctx context.Context, metadata Metadata) error
	Close(ctx context.Context) error
	Status(ctx context.Context) ComponentStatus
}
