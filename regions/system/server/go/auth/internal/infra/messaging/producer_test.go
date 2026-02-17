package messaging

import (
	"context"
	"encoding/json"
	"errors"
	"testing"
	"time"

	"github.com/k1s0-platform/system-server-go-auth/internal/domain/model"
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

func makeTestAuditLog() *model.AuditLog {
	return &model.AuditLog{
		ID:         "test-uuid-1234",
		EventType:  "LOGIN_SUCCESS",
		UserID:     "user-uuid-5678",
		IPAddress:  "192.168.1.100",
		UserAgent:  "Mozilla/5.0",
		Resource:   "/api/v1/auth/token",
		Action:     "POST",
		Result:     "SUCCESS",
		Metadata:   map[string]string{"client_id": "react-spa"},
		RecordedAt: time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC),
	}
}

func TestPublish_Serialization(t *testing.T) {
	mock := &mockWriter{}
	p := &KafkaProducer{
		writer: mock,
		topic:  "audit-events",
	}

	log := makeTestAuditLog()
	err := p.Publish(context.Background(), log)
	require.NoError(t, err)

	require.Len(t, mock.messages, 1)
	msg := mock.messages[0]

	// JSON に正常変換されていることを確認
	var deserialized model.AuditLog
	err = json.Unmarshal(msg.Value, &deserialized)
	require.NoError(t, err)
	assert.Equal(t, log.ID, deserialized.ID)
	assert.Equal(t, log.EventType, deserialized.EventType)
	assert.Equal(t, log.UserID, deserialized.UserID)
	assert.Equal(t, log.Result, deserialized.Result)
	assert.Equal(t, log.Metadata["client_id"], deserialized.Metadata["client_id"])
}

func TestPublish_KeyIsUserID(t *testing.T) {
	mock := &mockWriter{}
	p := &KafkaProducer{
		writer: mock,
		topic:  "audit-events",
	}

	log := makeTestAuditLog()
	err := p.Publish(context.Background(), log)
	require.NoError(t, err)

	require.Len(t, mock.messages, 1)
	// パーティションキーが user_id であることを確認
	assert.Equal(t, []byte(log.UserID), mock.messages[0].Key)
}

func TestPublish_ConnectionError(t *testing.T) {
	mock := &mockWriter{
		err: errors.New("broker connection refused"),
	}
	p := &KafkaProducer{
		writer: mock,
		topic:  "audit-events",
	}

	log := makeTestAuditLog()
	err := p.Publish(context.Background(), log)
	require.Error(t, err)
	assert.Contains(t, err.Error(), "broker connection refused")
}

func TestClose_Graceful(t *testing.T) {
	mock := &mockWriter{}
	p := &KafkaProducer{
		writer: mock,
		topic:  "audit-events",
	}

	err := p.Close()
	require.NoError(t, err)
	assert.True(t, mock.closed)
}
