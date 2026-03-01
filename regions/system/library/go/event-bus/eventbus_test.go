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

// --- レガシー API テスト ---

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

// --- DDD パターン テスト用 DomainEvent 実装 ---

type UserCreatedEvent struct {
	UserID    string
	Name      string
	CreatedAt time.Time
}

func (e UserCreatedEvent) EventType() string    { return "user.created" }
func (e UserCreatedEvent) AggregateID() string  { return e.UserID }
func (e UserCreatedEvent) OccurredAt() time.Time { return e.CreatedAt }

type OrderPlacedEvent struct {
	OrderID   string
	Amount    float64
	CreatedAt time.Time
}

func (e OrderPlacedEvent) EventType() string    { return "order.placed" }
func (e OrderPlacedEvent) AggregateID() string  { return e.OrderID }
func (e OrderPlacedEvent) OccurredAt() time.Time { return e.CreatedAt }

// --- DDD テスト用ハンドラー ---

type userCreatedHandler struct {
	received []UserCreatedEvent
}

func (h *userCreatedHandler) Handle(_ context.Context, event UserCreatedEvent) error {
	h.received = append(h.received, event)
	return nil
}

// --- DDD パターン テスト ---

func TestDomainEvent_Interface(t *testing.T) {
	now := time.Now()
	event := UserCreatedEvent{UserID: "user-123", Name: "alice", CreatedAt: now}

	// DomainEvent インターフェースを満たすことを確認
	var de eventbus.DomainEvent = event
	assert.Equal(t, "user.created", de.EventType())
	assert.Equal(t, "user-123", de.AggregateID())
	assert.Equal(t, now, de.OccurredAt())
}

func TestEventBusConfig_Default(t *testing.T) {
	config := eventbus.DefaultEventBusConfig()
	assert.Equal(t, 1024, config.BufferSize)
	assert.Equal(t, 30*time.Second, config.HandlerTimeout)
}

func TestNewEventBus(t *testing.T) {
	config := eventbus.EventBusConfig{
		BufferSize:     2048,
		HandlerTimeout: 60 * time.Second,
	}
	bus := eventbus.NewEventBus(config)
	assert.NotNil(t, bus)
}

func TestEventBus_SubscribeType_And_Publish(t *testing.T) {
	bus := eventbus.NewEventBus(eventbus.DefaultEventBusConfig())

	handler := &userCreatedHandler{}
	sub := eventbus.SubscribeType[UserCreatedEvent](bus, "user.created", handler)
	defer sub.Unsubscribe()

	event := UserCreatedEvent{
		UserID:    "user-123",
		Name:      "alice",
		CreatedAt: time.Now(),
	}

	err := eventbus.Publish(context.Background(), bus, event)
	require.NoError(t, err)
	assert.Len(t, handler.received, 1)
	assert.Equal(t, "user-123", handler.received[0].UserID)
	assert.Equal(t, "alice", handler.received[0].Name)
}

func TestEventBus_Subscribe_Wildcard(t *testing.T) {
	bus := eventbus.NewEventBus(eventbus.DefaultEventBusConfig())

	handler := &userCreatedHandler{}
	sub := eventbus.Subscribe[UserCreatedEvent](bus, handler)
	defer sub.Unsubscribe()

	event := UserCreatedEvent{
		UserID:    "user-456",
		Name:      "bob",
		CreatedAt: time.Now(),
	}

	err := eventbus.Publish(context.Background(), bus, event)
	require.NoError(t, err)
	assert.Len(t, handler.received, 1)
	assert.Equal(t, "user-456", handler.received[0].UserID)
}

func TestEventBus_Unsubscribe(t *testing.T) {
	bus := eventbus.NewEventBus(eventbus.DefaultEventBusConfig())

	handler := &userCreatedHandler{}
	sub := eventbus.SubscribeType[UserCreatedEvent](bus, "user.created", handler)

	// 購読解除
	sub.Unsubscribe()

	event := UserCreatedEvent{
		UserID:    "user-789",
		Name:      "charlie",
		CreatedAt: time.Now(),
	}

	err := eventbus.Publish(context.Background(), bus, event)
	require.NoError(t, err)
	assert.Len(t, handler.received, 0)
}

func TestEventBus_MultipleHandlers(t *testing.T) {
	bus := eventbus.NewEventBus(eventbus.DefaultEventBusConfig())

	handler1 := &userCreatedHandler{}
	handler2 := &userCreatedHandler{}
	sub1 := eventbus.SubscribeType[UserCreatedEvent](bus, "user.created", handler1)
	sub2 := eventbus.SubscribeType[UserCreatedEvent](bus, "user.created", handler2)
	defer sub1.Unsubscribe()
	defer sub2.Unsubscribe()

	event := UserCreatedEvent{UserID: "user-1", Name: "alice", CreatedAt: time.Now()}
	err := eventbus.Publish(context.Background(), bus, event)
	require.NoError(t, err)

	assert.Len(t, handler1.received, 1)
	assert.Len(t, handler2.received, 1)
}

func TestEventBus_NoSubscribers(t *testing.T) {
	bus := eventbus.NewEventBus(eventbus.DefaultEventBusConfig())

	event := UserCreatedEvent{UserID: "user-1", Name: "alice", CreatedAt: time.Now()}
	err := eventbus.Publish(context.Background(), bus, event)
	assert.NoError(t, err)
}

func TestEventBus_HandlerError(t *testing.T) {
	bus := eventbus.NewEventBus(eventbus.DefaultEventBusConfig())

	failHandler := eventbus.EventHandlerFunc[UserCreatedEvent](
		func(_ context.Context, _ UserCreatedEvent) error {
			return errors.New("handler error")
		},
	)
	sub := eventbus.SubscribeType[UserCreatedEvent](bus, "user.created", failHandler)
	defer sub.Unsubscribe()

	event := UserCreatedEvent{UserID: "user-1", Name: "alice", CreatedAt: time.Now()}
	err := eventbus.Publish(context.Background(), bus, event)
	assert.Error(t, err)

	var busErr *eventbus.EventBusError
	assert.True(t, errors.As(err, &busErr))
	assert.Equal(t, eventbus.HandlerFailed, busErr.Kind)
}

func TestEventBus_HandlerTimeout(t *testing.T) {
	config := eventbus.EventBusConfig{
		BufferSize:     1024,
		HandlerTimeout: 50 * time.Millisecond,
	}
	bus := eventbus.NewEventBus(config)

	slowHandler := eventbus.EventHandlerFunc[UserCreatedEvent](
		func(ctx context.Context, _ UserCreatedEvent) error {
			select {
			case <-time.After(2 * time.Second):
				return nil
			case <-ctx.Done():
				return ctx.Err()
			}
		},
	)
	sub := eventbus.SubscribeType[UserCreatedEvent](bus, "user.created", slowHandler)
	defer sub.Unsubscribe()

	event := UserCreatedEvent{UserID: "user-1", Name: "alice", CreatedAt: time.Now()}
	err := eventbus.Publish(context.Background(), bus, event)
	assert.Error(t, err)

	var busErr *eventbus.EventBusError
	assert.True(t, errors.As(err, &busErr))
	assert.Equal(t, eventbus.HandlerFailed, busErr.Kind)
}

func TestEventBus_DifferentEventTypes_Isolated(t *testing.T) {
	bus := eventbus.NewEventBus(eventbus.DefaultEventBusConfig())

	userHandler := &userCreatedHandler{}
	orderCount := 0
	orderHandler := eventbus.EventHandlerFunc[OrderPlacedEvent](
		func(_ context.Context, _ OrderPlacedEvent) error {
			orderCount++
			return nil
		},
	)

	sub1 := eventbus.SubscribeType[UserCreatedEvent](bus, "user.created", userHandler)
	sub2 := eventbus.SubscribeType[OrderPlacedEvent](bus, "order.placed", orderHandler)
	defer sub1.Unsubscribe()
	defer sub2.Unsubscribe()

	// user.created を発行 → userHandler のみ呼ばれる
	userEvent := UserCreatedEvent{UserID: "user-1", Name: "alice", CreatedAt: time.Now()}
	err := eventbus.Publish(context.Background(), bus, userEvent)
	require.NoError(t, err)

	assert.Len(t, userHandler.received, 1)
	assert.Equal(t, 0, orderCount)
}

func TestEventBusError_Formatting(t *testing.T) {
	err := &eventbus.EventBusError{Kind: eventbus.PublishFailed, Message: "test error"}
	assert.Equal(t, "publish failed: test error", err.Error())

	err2 := &eventbus.EventBusError{Kind: eventbus.HandlerFailed, Message: "handler error"}
	assert.Equal(t, "handler failed: handler error", err2.Error())

	err3 := &eventbus.EventBusError{Kind: eventbus.ChannelClosed}
	assert.Equal(t, "channel closed", err3.Error())
}

func TestEventHandlerFunc_Adapter(t *testing.T) {
	var received UserCreatedEvent
	handler := eventbus.EventHandlerFunc[UserCreatedEvent](
		func(_ context.Context, event UserCreatedEvent) error {
			received = event
			return nil
		},
	)

	event := UserCreatedEvent{UserID: "user-1", Name: "alice", CreatedAt: time.Now()}
	err := handler.Handle(context.Background(), event)
	require.NoError(t, err)
	assert.Equal(t, "user-1", received.UserID)
}
