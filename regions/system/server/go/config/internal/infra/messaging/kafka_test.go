package messaging

import (
	"context"
	"encoding/json"
	"errors"
	"testing"
	"time"

	"github.com/k1s0-platform/system-server-go-config/internal/domain/model"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// mockWriter は kafka.Writer のモック実装。
type mockWriter struct {
	messages []writerMessage
	err      error
	closed   bool
}

func (m *mockWriter) WriteMessages(ctx context.Context, msgs ...writerMessage) error {
	if m.err != nil {
		return m.err
	}
	m.messages = append(m.messages, msgs...)
	return nil
}

func (m *mockWriter) Close() error {
	m.closed = true
	return nil
}

func makeTestConfigChangeLog() *model.ConfigChangeLog {
	oldValue, _ := json.Marshal(25)
	newValue, _ := json.Marshal(50)
	return &model.ConfigChangeLog{
		ID:            "test-uuid-1234",
		ConfigEntryID: "entry-uuid-5678",
		Namespace:     "system.auth.database",
		Key:           "max_connections",
		OldValue:      oldValue,
		NewValue:      newValue,
		OldVersion:    3,
		NewVersion:    4,
		ChangeType:    "UPDATED",
		ChangedBy:     "operator@example.com",
		ChangedAt:     time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC),
	}
}

func TestPublish_Serialization(t *testing.T) {
	mock := &mockWriter{}
	p := &KafkaProducer{
		writer: mock,
		topic:  "k1s0.system.config.changed.v1",
	}

	log := makeTestConfigChangeLog()
	err := p.Publish(context.Background(), log)
	require.NoError(t, err)

	require.Len(t, mock.messages, 1)
	msg := mock.messages[0]

	// JSON に正常変換されていることを確認
	var deserialized model.ConfigChangeLog
	err = json.Unmarshal(msg.Value, &deserialized)
	require.NoError(t, err)
	assert.Equal(t, log.ID, deserialized.ID)
	assert.Equal(t, log.Namespace, deserialized.Namespace)
	assert.Equal(t, log.Key, deserialized.Key)
	assert.Equal(t, log.ChangeType, deserialized.ChangeType)
	assert.Equal(t, log.ChangedBy, deserialized.ChangedBy)
}

func TestPublish_KeyIsNamespaceKey(t *testing.T) {
	mock := &mockWriter{}
	p := &KafkaProducer{
		writer: mock,
		topic:  "k1s0.system.config.changed.v1",
	}

	log := makeTestConfigChangeLog()
	err := p.Publish(context.Background(), log)
	require.NoError(t, err)

	require.Len(t, mock.messages, 1)
	// パーティションキーが namespace.key であることを確認
	assert.Equal(t, []byte(log.Namespace+"."+log.Key), mock.messages[0].Key)
}

func TestPublish_TopicName(t *testing.T) {
	mock := &mockWriter{}
	p := &KafkaProducer{
		writer: mock,
		topic:  "k1s0.system.config.changed.v1",
	}

	log := makeTestConfigChangeLog()
	err := p.Publish(context.Background(), log)
	require.NoError(t, err)

	require.Len(t, mock.messages, 1)
	assert.Equal(t, "k1s0.system.config.changed.v1", mock.messages[0].Topic)
}

func TestPublish_ConnectionError(t *testing.T) {
	mock := &mockWriter{
		err: errors.New("broker connection refused"),
	}
	p := &KafkaProducer{
		writer: mock,
		topic:  "k1s0.system.config.changed.v1",
	}

	log := makeTestConfigChangeLog()
	err := p.Publish(context.Background(), log)
	require.Error(t, err)
	assert.Contains(t, err.Error(), "broker connection refused")
}

func TestClose_Graceful(t *testing.T) {
	mock := &mockWriter{}
	p := &KafkaProducer{
		writer: mock,
		topic:  "k1s0.system.config.changed.v1",
	}

	err := p.Close()
	require.NoError(t, err)
	assert.True(t, mock.closed)
}

func TestHealthy(t *testing.T) {
	mock := &mockWriter{}
	p := &KafkaProducer{
		writer: mock,
		topic:  "k1s0.system.config.changed.v1",
	}

	err := p.Healthy(context.Background())
	require.NoError(t, err)
}
