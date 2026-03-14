package buildingblocks

import (
	"encoding/json"
	"testing"
	"time"
)

func TestMessageCreation(t *testing.T) {
	now := time.Now()
	m := Message{
		Topic:     "orders",
		Data:      []byte(`{"id": 1}`),
		Metadata:  map[string]string{"source": "api"},
		ID:        "msg-001",
		Timestamp: now,
	}
	if m.Topic != "orders" {
		t.Errorf("expected Topic 'orders', got %q", m.Topic)
	}
	if string(m.Data) != `{"id": 1}` {
		t.Errorf("expected Data %q, got %q", `{"id": 1}`, string(m.Data))
	}
	if m.Metadata["source"] != "api" {
		t.Errorf("expected Metadata[source] %q, got %q", "api", m.Metadata["source"])
	}
	if m.ID != "msg-001" {
		t.Errorf("expected ID 'msg-001', got %q", m.ID)
	}
	if !m.Timestamp.Equal(now) {
		t.Errorf("expected Timestamp %v, got %v", now, m.Timestamp)
	}
}

func TestMessageJSONRoundtrip(t *testing.T) {
	now := time.Date(2025, 1, 15, 10, 30, 0, 0, time.UTC)
	m := Message{
		Topic:     "events",
		Data:      []byte("hello"),
		Metadata:  map[string]string{"key": "val"},
		ID:        "msg-002",
		Timestamp: now,
	}
	data, err := json.Marshal(m)
	if err != nil {
		t.Fatalf("failed to marshal Message: %v", err)
	}

	var decoded Message
	if err := json.Unmarshal(data, &decoded); err != nil {
		t.Fatalf("failed to unmarshal Message: %v", err)
	}
	if decoded.Topic != m.Topic || decoded.ID != m.ID {
		t.Errorf("roundtrip mismatch: got %+v", decoded)
	}
	if !decoded.Timestamp.Equal(now) {
		t.Errorf("expected Timestamp %v, got %v", now, decoded.Timestamp)
	}
	if string(decoded.Data) != "hello" {
		t.Errorf("expected Data 'hello', got %q", string(decoded.Data))
	}
}

func TestMessageJSONOmitEmptyMetadata(t *testing.T) {
	m := Message{Topic: "t", Data: []byte("d"), ID: "1", Timestamp: time.Now()}
	data, err := json.Marshal(m)
	if err != nil {
		t.Fatalf("failed to marshal: %v", err)
	}
	var raw map[string]any
	if err := json.Unmarshal(data, &raw); err != nil {
		t.Fatalf("failed to unmarshal: %v", err)
	}
	if _, ok := raw["metadata"]; ok {
		t.Error("expected 'metadata' to be omitted when nil")
	}
}
