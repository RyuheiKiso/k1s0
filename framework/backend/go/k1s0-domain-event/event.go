// Package domainevent provides domain event publishing, subscribing, and outbox pattern.
package domainevent

// DomainEvent is the base interface for all domain events.
type DomainEvent interface {
	// EventType returns the event type identifier (e.g., "order.created").
	EventType() string

	// AggregateID returns the aggregate root ID (optional).
	AggregateID() string

	// AggregateType returns the aggregate type (optional).
	AggregateType() string
}

// BaseDomainEvent provides a default implementation of DomainEvent.
type BaseDomainEvent struct {
	Type      string
	AggrID    string
	AggrType  string
}

func (e *BaseDomainEvent) EventType() string     { return e.Type }
func (e *BaseDomainEvent) AggregateID() string    { return e.AggrID }
func (e *BaseDomainEvent) AggregateType() string  { return e.AggrType }
