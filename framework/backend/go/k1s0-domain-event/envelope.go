package domainevent

import (
	"encoding/json"
	"time"

	"github.com/google/uuid"
)

// EventMetadata holds metadata associated with a domain event.
type EventMetadata struct {
	// EventID is a unique identifier for this event.
	EventID string `json:"event_id"`

	// OccurredAt is the time the event occurred.
	OccurredAt time.Time `json:"occurred_at"`

	// Source is the name of the service that produced the event.
	Source string `json:"source"`

	// CorrelationID is used for distributed tracing (optional).
	CorrelationID string `json:"correlation_id,omitempty"`

	// CausationID tracks causality (optional).
	CausationID string `json:"causation_id,omitempty"`
}

// NewEventMetadata creates a new EventMetadata with a generated ID and current time.
func NewEventMetadata(source string) EventMetadata {
	return EventMetadata{
		EventID:    uuid.New().String(),
		OccurredAt: time.Now().UTC(),
		Source:     source,
	}
}

// EventEnvelope wraps an event payload with metadata.
type EventEnvelope struct {
	// EventType is the event type identifier.
	EventType string `json:"event_type"`

	// Metadata contains event metadata.
	Metadata EventMetadata `json:"metadata"`

	// Payload is the JSON-encoded event body.
	Payload json.RawMessage `json:"payload"`
}

// NewEventEnvelope creates an EventEnvelope from a DomainEvent.
func NewEventEnvelope(event DomainEvent, source string) (*EventEnvelope, error) {
	payload, err := json.Marshal(event)
	if err != nil {
		return nil, err
	}
	return &EventEnvelope{
		EventType: event.EventType(),
		Metadata:  NewEventMetadata(source),
		Payload:   payload,
	}, nil
}
