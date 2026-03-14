package buildingblocks

import (
	"context"
	"sync"
	"time"
)

// Compile-time interface compliance check.
var _ PubSub = (*InMemoryPubSub)(nil)

// InMemoryPubSub is an in-memory implementation of PubSub for testing.
type InMemoryPubSub struct {
	mu      sync.RWMutex
	subs    map[string][]chan *Message
	status  ComponentStatus
}

// NewInMemoryPubSub creates a new InMemoryPubSub.
func NewInMemoryPubSub() *InMemoryPubSub {
	return &InMemoryPubSub{
		subs:   make(map[string][]chan *Message),
		status: StatusUninitialized,
	}
}

func (p *InMemoryPubSub) Name() string    { return "inmemory-pubsub" }
func (p *InMemoryPubSub) Version() string { return "1.0.0" }

func (p *InMemoryPubSub) Init(_ context.Context, _ Metadata) error {
	p.mu.Lock()
	defer p.mu.Unlock()
	p.status = StatusReady
	return nil
}

func (p *InMemoryPubSub) Close(_ context.Context) error {
	p.mu.Lock()
	defer p.mu.Unlock()
	for _, chans := range p.subs {
		for _, ch := range chans {
			close(ch)
		}
	}
	p.subs = make(map[string][]chan *Message)
	p.status = StatusClosed
	return nil
}

func (p *InMemoryPubSub) Status(_ context.Context) ComponentStatus {
	p.mu.RLock()
	defer p.mu.RUnlock()
	return p.status
}

// Publish sends msg to all subscribers of msg.Topic (non-blocking).
// If msg.Timestamp is zero, the sent copy has its Timestamp set to now.
func (p *InMemoryPubSub) Publish(_ context.Context, msg *Message) error {
	out := *msg
	if out.Timestamp.IsZero() {
		out.Timestamp = time.Now()
	}
	p.mu.RLock()
	defer p.mu.RUnlock()
	for _, ch := range p.subs[out.Topic] {
		select {
		case ch <- &out:
		default:
		}
	}
	return nil
}

// Subscribe returns a channel that receives messages published to topic.
func (p *InMemoryPubSub) Subscribe(_ context.Context, topic string) (<-chan *Message, error) {
	ch := make(chan *Message, 64)
	p.mu.Lock()
	defer p.mu.Unlock()
	p.subs[topic] = append(p.subs[topic], ch)
	return ch, nil
}
