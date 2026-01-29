// Package bus provides event bus implementations.
package bus

import (
	"context"
	"sync"

	domainevent "github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-domain-event"
)

// InMemoryEventBus is a process-local event bus for testing and single-process use.
type InMemoryEventBus struct {
	mu       sync.RWMutex
	handlers map[string][]subscription
	nextID   int
}

type subscription struct {
	id      int
	handler domainevent.EventHandler
}

// NewInMemoryEventBus creates a new InMemoryEventBus.
func NewInMemoryEventBus() *InMemoryEventBus {
	return &InMemoryEventBus{
		handlers: make(map[string][]subscription),
	}
}

// Publish sends an event to all matching handlers synchronously.
func (b *InMemoryEventBus) Publish(ctx context.Context, envelope *domainevent.EventEnvelope) error {
	b.mu.RLock()
	subs := b.handlers[envelope.EventType]
	b.mu.RUnlock()

	for _, sub := range subs {
		if err := sub.handler.Handle(ctx, envelope); err != nil {
			return &domainevent.PublishError{Cause: err, Msg: "handler failed during publish"}
		}
	}
	return nil
}

// PublishBatch sends multiple events sequentially.
func (b *InMemoryEventBus) PublishBatch(ctx context.Context, envelopes []*domainevent.EventEnvelope) error {
	for _, env := range envelopes {
		if err := b.Publish(ctx, env); err != nil {
			return err
		}
	}
	return nil
}

// Subscribe registers a handler and returns a cancel function.
func (b *InMemoryEventBus) Subscribe(handler domainevent.EventHandler) (func(), error) {
	b.mu.Lock()
	defer b.mu.Unlock()

	id := b.nextID
	b.nextID++
	eventType := handler.EventType()

	b.handlers[eventType] = append(b.handlers[eventType], subscription{
		id:      id,
		handler: handler,
	})

	cancel := func() {
		b.mu.Lock()
		defer b.mu.Unlock()
		subs := b.handlers[eventType]
		for i, s := range subs {
			if s.id == id {
				b.handlers[eventType] = append(subs[:i], subs[i+1:]...)
				break
			}
		}
	}

	return cancel, nil
}
