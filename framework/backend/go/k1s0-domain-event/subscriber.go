package domainevent

import "context"

// EventHandler processes domain events of a specific type.
type EventHandler interface {
	// EventType returns the event type this handler processes.
	EventType() string

	// Handle processes the event envelope.
	Handle(ctx context.Context, envelope *EventEnvelope) error
}

// EventSubscriber manages event subscriptions.
type EventSubscriber interface {
	// Subscribe registers a handler and returns a function to cancel the subscription.
	Subscribe(handler EventHandler) (cancel func(), err error)
}
