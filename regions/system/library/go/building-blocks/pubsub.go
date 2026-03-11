package buildingblocks

import "context"

// Message represents a pub/sub message.
type Message struct {
	Topic    string
	Data     []byte
	Metadata map[string]string
	ID       string
}

// MessageHandler processes incoming messages.
type MessageHandler interface {
	Handle(ctx context.Context, msg Message) error
}

// PubSub provides publish/subscribe messaging capabilities.
type PubSub interface {
	Component
	Publish(ctx context.Context, topic string, data []byte, metadata map[string]string) error
	Subscribe(ctx context.Context, topic string, handler MessageHandler) (string, error)
	Unsubscribe(ctx context.Context, subscriptionID string) error
}
