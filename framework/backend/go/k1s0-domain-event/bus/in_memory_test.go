package bus_test

import (
	"context"
	"sync/atomic"
	"testing"

	domainevent "github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-domain-event"
	"github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-domain-event/bus"
)

type testHandler struct {
	eventType string
	count     *atomic.Int32
}

func (h *testHandler) EventType() string { return h.eventType }
func (h *testHandler) Handle(_ context.Context, _ *domainevent.EventEnvelope) error {
	h.count.Add(1)
	return nil
}

func TestPublishAndSubscribe(t *testing.T) {
	b := bus.NewInMemoryEventBus()
	count := &atomic.Int32{}

	_, err := b.Subscribe(&testHandler{eventType: "test.event", count: count})
	if err != nil {
		t.Fatalf("subscribe failed: %v", err)
	}

	envelope := &domainevent.EventEnvelope{
		EventType: "test.event",
		Metadata:  domainevent.NewEventMetadata("test"),
		Payload:   []byte(`{}`),
	}

	if err := b.Publish(context.Background(), envelope); err != nil {
		t.Fatalf("publish failed: %v", err)
	}

	if got := count.Load(); got != 1 {
		t.Errorf("expected count 1, got %d", got)
	}
}

func TestUnmatchedEventIgnored(t *testing.T) {
	b := bus.NewInMemoryEventBus()
	count := &atomic.Int32{}

	_, err := b.Subscribe(&testHandler{eventType: "test.event", count: count})
	if err != nil {
		t.Fatalf("subscribe failed: %v", err)
	}

	envelope := &domainevent.EventEnvelope{
		EventType: "other.event",
		Metadata:  domainevent.NewEventMetadata("test"),
		Payload:   []byte(`{}`),
	}

	if err := b.Publish(context.Background(), envelope); err != nil {
		t.Fatalf("publish failed: %v", err)
	}

	if got := count.Load(); got != 0 {
		t.Errorf("expected count 0, got %d", got)
	}
}

func TestCancelSubscription(t *testing.T) {
	b := bus.NewInMemoryEventBus()
	count := &atomic.Int32{}

	cancel, err := b.Subscribe(&testHandler{eventType: "test.event", count: count})
	if err != nil {
		t.Fatalf("subscribe failed: %v", err)
	}

	cancel()

	envelope := &domainevent.EventEnvelope{
		EventType: "test.event",
		Metadata:  domainevent.NewEventMetadata("test"),
		Payload:   []byte(`{}`),
	}

	if err := b.Publish(context.Background(), envelope); err != nil {
		t.Fatalf("publish failed: %v", err)
	}

	if got := count.Load(); got != 0 {
		t.Errorf("expected count 0 after cancel, got %d", got)
	}
}
