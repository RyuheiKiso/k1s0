package eventbus_test

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/k1s0-platform/system-library-go-event-bus"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestSubscribe_And_Publish(t *testing.T) {
	bus := eventbus.New()
	var received []eventbus.Event

	bus.Subscribe("user.created", func(_ context.Context, e eventbus.Event) error {
		received = append(received, e)
		return nil
	})

	evt := eventbus.Event{
		ID:        "1",
		EventType: "user.created",
		Payload:   map[string]any{"name": "alice"},
		Timestamp: time.Now(),
	}
	err := bus.Publish(context.Background(), evt)
	require.NoError(t, err)
	assert.Len(t, received, 1)
	assert.Equal(t, "1", received[0].ID)
}

func TestPublish_NoSubscribers(t *testing.T) {
	bus := eventbus.New()
	evt := eventbus.Event{ID: "1", EventType: "unknown"}
	err := bus.Publish(context.Background(), evt)
	assert.NoError(t, err)
}

func TestUnsubscribe(t *testing.T) {
	bus := eventbus.New()
	called := false
	bus.Subscribe("test.event", func(_ context.Context, _ eventbus.Event) error {
		called = true
		return nil
	})
	bus.Unsubscribe("test.event")

	_ = bus.Publish(context.Background(), eventbus.Event{EventType: "test.event"})
	assert.False(t, called)
}

func TestMultipleHandlers(t *testing.T) {
	bus := eventbus.New()
	count := 0

	bus.Subscribe("evt", func(_ context.Context, _ eventbus.Event) error { count++; return nil })
	bus.Subscribe("evt", func(_ context.Context, _ eventbus.Event) error { count++; return nil })

	_ = bus.Publish(context.Background(), eventbus.Event{EventType: "evt"})
	assert.Equal(t, 2, count)
}

func TestPublish_HandlerError(t *testing.T) {
	bus := eventbus.New()
	bus.Subscribe("evt", func(_ context.Context, _ eventbus.Event) error {
		return errors.New("handler error")
	})
	err := bus.Publish(context.Background(), eventbus.Event{EventType: "evt"})
	assert.Error(t, err)
}

func TestDifferentEventTypes_Isolated(t *testing.T) {
	bus := eventbus.New()
	var typeA, typeB int

	bus.Subscribe("type.a", func(_ context.Context, _ eventbus.Event) error { typeA++; return nil })
	bus.Subscribe("type.b", func(_ context.Context, _ eventbus.Event) error { typeB++; return nil })

	_ = bus.Publish(context.Background(), eventbus.Event{EventType: "type.a"})
	assert.Equal(t, 1, typeA)
	assert.Equal(t, 0, typeB)
}
