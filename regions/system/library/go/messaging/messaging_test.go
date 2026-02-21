package messaging_test

import (
	"context"
	"errors"
	"testing"

	messaging "github.com/k1s0-platform/system-library-go-messaging"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestNewEventMetadata(t *testing.T) {
	meta := messaging.NewEventMetadata("user.created.v1", "corr-123", "auth-service")
	assert.NotEmpty(t, meta.EventId)
	assert.Equal(t, "user.created.v1", meta.EventType)
	assert.Equal(t, "corr-123", meta.CorrelationId)
	assert.Equal(t, "auth-service", meta.Source)
	assert.False(t, meta.Timestamp.IsZero())
}

func TestNewEventMetadata_UniqueIds(t *testing.T) {
	meta1 := messaging.NewEventMetadata("event.v1", "corr-1", "svc")
	meta2 := messaging.NewEventMetadata("event.v1", "corr-1", "svc")
	assert.NotEqual(t, meta1.EventId, meta2.EventId)
}

func TestNoOpEventProducer_Publish(t *testing.T) {
	producer := &messaging.NoOpEventProducer{}
	event := messaging.EventEnvelope{
		Metadata: messaging.NewEventMetadata("test.v1", "corr-1", "svc"),
		Topic:    "k1s0.system.test.event.v1",
		Payload:  map[string]string{"key": "value"},
	}
	err := producer.Publish(context.Background(), event)
	require.NoError(t, err)
	assert.Equal(t, 1, producer.PublishedCount())
}

func TestNoOpEventProducer_Publish_Multiple(t *testing.T) {
	producer := &messaging.NoOpEventProducer{}
	for i := 0; i < 3; i++ {
		err := producer.Publish(context.Background(), messaging.EventEnvelope{
			Metadata: messaging.NewEventMetadata("test.v1", "corr-1", "svc"),
			Topic:    "k1s0.system.test.event.v1",
		})
		require.NoError(t, err)
	}
	assert.Equal(t, 3, producer.PublishedCount())
}

func TestNoOpEventProducer_Publish_WithError(t *testing.T) {
	expectedErr := errors.New("publish failed")
	producer := &messaging.NoOpEventProducer{Err: expectedErr}
	err := producer.Publish(context.Background(), messaging.EventEnvelope{})
	assert.ErrorIs(t, err, expectedErr)
	assert.Equal(t, 0, producer.PublishedCount())
}

func TestNoOpEventProducer_Close(t *testing.T) {
	producer := &messaging.NoOpEventProducer{}
	assert.False(t, producer.IsClosed())
	err := producer.Close()
	require.NoError(t, err)
	assert.True(t, producer.IsClosed())
}

func TestMessagingError(t *testing.T) {
	cause := errors.New("connection refused")
	err := &messaging.MessagingError{Op: "Publish", Err: cause}
	assert.Contains(t, err.Error(), "Publish")
	assert.Contains(t, err.Error(), "connection refused")
	assert.ErrorIs(t, err, cause)
}

func TestEventEnvelope_WithHeaders(t *testing.T) {
	event := messaging.EventEnvelope{
		Metadata: messaging.NewEventMetadata("test.v1", "corr-1", "svc"),
		Topic:    "k1s0.system.test.event.v1",
		Headers: map[string]string{
			"X-Correlation-Id": "corr-1",
		},
	}
	assert.Equal(t, "corr-1", event.Headers["X-Correlation-Id"])
}

func TestNoOpEventProducer_RecordsPublishedEvents(t *testing.T) {
	producer := &messaging.NoOpEventProducer{}
	event := messaging.EventEnvelope{
		Metadata: messaging.NewEventMetadata("user.created.v1", "corr-1", "auth-service"),
		Topic:    "k1s0.system.user.created.v1",
		Payload:  "test-payload",
	}
	err := producer.Publish(context.Background(), event)
	require.NoError(t, err)
	require.Len(t, producer.Published, 1)
	assert.Equal(t, "k1s0.system.user.created.v1", producer.Published[0].Topic)
	assert.Equal(t, "test-payload", producer.Published[0].Payload)
}

func TestEventProducer_InterfaceCompliance(t *testing.T) {
	// NoOpEventProducer は EventProducer インターフェースを実装していることを確認
	var _ messaging.EventProducer = &messaging.NoOpEventProducer{}
}
