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

// Component is the base interface for all building block components.
type Component interface {
	Name() string
	Type() string
	Init(ctx context.Context) error
	Close(ctx context.Context) error
	Status(ctx context.Context) ComponentStatus
	Metadata() map[string]string
}
