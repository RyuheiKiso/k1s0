// Package outbox provides the transactional outbox pattern for domain events.
package outbox

import (
	"encoding/json"
	"time"

	"github.com/google/uuid"
)

// Status represents the state of an outbox entry.
type Status string

const (
	StatusPending   Status = "pending"
	StatusPublished Status = "published"
	StatusFailed    Status = "failed"
)

// Entry represents a single row in the outbox table.
type Entry struct {
	ID          string          `json:"id"`
	EventType   string          `json:"event_type"`
	Payload     json.RawMessage `json:"payload"`
	Status      Status          `json:"status"`
	CreatedAt   time.Time       `json:"created_at"`
	PublishedAt *time.Time      `json:"published_at,omitempty"`
	RetryCount  int             `json:"retry_count"`
	LastError   string          `json:"last_error,omitempty"`
}

// NewEntry creates a new pending outbox entry.
func NewEntry(eventType string, payload json.RawMessage) *Entry {
	return &Entry{
		ID:         uuid.New().String(),
		EventType:  eventType,
		Payload:    payload,
		Status:     StatusPending,
		CreatedAt:  time.Now().UTC(),
		RetryCount: 0,
	}
}
