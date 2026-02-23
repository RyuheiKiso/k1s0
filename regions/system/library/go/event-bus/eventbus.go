package eventbus

import (
	"context"
	"sync"
	"time"
)

// Event はイベント。
type Event struct {
	ID        string         `json:"id"`
	EventType string         `json:"event_type"`
	Payload   map[string]any `json:"payload"`
	Timestamp time.Time      `json:"timestamp"`
}

// Handler はイベントハンドラー。
type Handler func(ctx context.Context, event Event) error

// EventBus はイベントバスのインターフェース。
type EventBus interface {
	Subscribe(eventType string, handler Handler)
	Publish(ctx context.Context, event Event) error
	Unsubscribe(eventType string)
}

// InMemoryBus はメモリ内のイベントバス。
type InMemoryBus struct {
	mu       sync.RWMutex
	handlers map[string][]Handler
}

// New は新しい InMemoryBus を生成する。
func New() *InMemoryBus {
	return &InMemoryBus{
		handlers: make(map[string][]Handler),
	}
}

func (b *InMemoryBus) Subscribe(eventType string, handler Handler) {
	b.mu.Lock()
	defer b.mu.Unlock()
	b.handlers[eventType] = append(b.handlers[eventType], handler)
}

func (b *InMemoryBus) Publish(ctx context.Context, event Event) error {
	b.mu.RLock()
	handlers := b.handlers[event.EventType]
	b.mu.RUnlock()

	for _, h := range handlers {
		if err := h(ctx, event); err != nil {
			return err
		}
	}
	return nil
}

func (b *InMemoryBus) Unsubscribe(eventType string) {
	b.mu.Lock()
	defer b.mu.Unlock()
	delete(b.handlers, eventType)
}
