package domainevent_test

import (
	"testing"

	domainevent "github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-domain-event"
)

type orderCreated struct {
	domainevent.BaseDomainEvent
	OrderID string `json:"order_id"`
}

func TestNewEventEnvelope(t *testing.T) {
	event := &orderCreated{
		BaseDomainEvent: domainevent.BaseDomainEvent{Type: "order.created", AggrID: "ord-1"},
		OrderID:         "ord-1",
	}

	env, err := domainevent.NewEventEnvelope(event, "order-service")
	if err != nil {
		t.Fatalf("NewEventEnvelope failed: %v", err)
	}

	if env.EventType != "order.created" {
		t.Errorf("expected event type 'order.created', got %q", env.EventType)
	}
	if env.Metadata.Source != "order-service" {
		t.Errorf("expected source 'order-service', got %q", env.Metadata.Source)
	}
	if len(env.Payload) == 0 {
		t.Error("expected non-empty payload")
	}
}
