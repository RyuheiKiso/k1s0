package buildingblocks

import (
	"context"
	"time"
)

// Message represents a pub/sub message.
type Message struct {
	Topic     string            `json:"topic"`
	Data      []byte            `json:"data"`
	Metadata  map[string]string `json:"metadata,omitempty"`
	ID        string            `json:"id"`
	Timestamp time.Time         `json:"timestamp"`
}

// PubSub provides publish/subscribe messaging capabilities.
type PubSub interface {
	Component
	Publish(ctx context.Context, msg *Message) error
	Subscribe(ctx context.Context, topic string) (<-chan *Message, error)
}
