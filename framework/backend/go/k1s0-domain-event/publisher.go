package domainevent

import "context"

// EventPublisher publishes domain event envelopes.
type EventPublisher interface {
	// Publish sends an event envelope.
	Publish(ctx context.Context, envelope *EventEnvelope) error

	// PublishBatch sends multiple event envelopes.
	PublishBatch(ctx context.Context, envelopes []*EventEnvelope) error
}
