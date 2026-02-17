package messaging

import (
	"context"
	"encoding/json"
	"testing"

	"github.com/k1s0-platform/system-server-go-auth/internal/domain/model"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestRoundTrip_ProducerConsumer はプロデューサーが送信したメッセージを
// コンシューマー側で正しくデシリアライズできることを検証する。
func TestRoundTrip_ProducerConsumer(t *testing.T) {
	mock := &mockWriter{}
	p := &KafkaProducer{
		writer: mock,
		topic:  "k1s0.system.auth.audit.v1",
	}

	log := makeTestAuditLog()
	err := p.Publish(context.Background(), log)
	require.NoError(t, err)

	require.Len(t, mock.messages, 1)
	msg := mock.messages[0]

	// コンシューマー側のデシリアライズをシミュレート
	var consumed model.AuditLog
	err = json.Unmarshal(msg.Value, &consumed)
	require.NoError(t, err)

	// 全フィールドが正しく復元されることを確認
	assert.Equal(t, log.ID, consumed.ID)
	assert.Equal(t, log.EventType, consumed.EventType)
	assert.Equal(t, log.UserID, consumed.UserID)
	assert.Equal(t, log.IPAddress, consumed.IPAddress)
	assert.Equal(t, log.UserAgent, consumed.UserAgent)
	assert.Equal(t, log.Resource, consumed.Resource)
	assert.Equal(t, log.Action, consumed.Action)
	assert.Equal(t, log.Result, consumed.Result)
	assert.Equal(t, log.Metadata, consumed.Metadata)
	assert.Equal(t, log.RecordedAt, consumed.RecordedAt)
}

// TestRoundTrip_MultipleEvents は複数のイベントを連続送信した場合の
// デシリアライズを検証する。
func TestRoundTrip_MultipleEvents(t *testing.T) {
	mock := &mockWriter{}
	p := &KafkaProducer{
		writer: mock,
		topic:  "k1s0.system.auth.audit.v1",
	}

	events := []*model.AuditLog{
		{
			ID:        "id-1",
			EventType: "LOGIN_SUCCESS",
			UserID:    "user-1",
			IPAddress: "10.0.0.1",
			Resource:  "/auth/token",
			Action:    "POST",
			Result:    "SUCCESS",
			Metadata:  map[string]string{},
		},
		{
			ID:        "id-2",
			EventType: "LOGIN_FAILURE",
			UserID:    "user-2",
			IPAddress: "10.0.0.2",
			Resource:  "/auth/token",
			Action:    "POST",
			Result:    "FAILURE",
			Metadata:  map[string]string{"reason": "invalid_password"},
		},
		{
			ID:        "id-3",
			EventType: "TOKEN_VALIDATE",
			UserID:    "user-1",
			IPAddress: "10.0.0.1",
			Resource:  "/auth/validate",
			Action:    "POST",
			Result:    "SUCCESS",
			Metadata:  map[string]string{},
		},
	}

	for _, e := range events {
		err := p.Publish(context.Background(), e)
		require.NoError(t, err)
	}

	require.Len(t, mock.messages, 3)

	for i, msg := range mock.messages {
		var consumed model.AuditLog
		err := json.Unmarshal(msg.Value, &consumed)
		require.NoError(t, err)
		assert.Equal(t, events[i].ID, consumed.ID)
		assert.Equal(t, events[i].EventType, consumed.EventType)
		assert.Equal(t, events[i].UserID, consumed.UserID)
		// パーティションキーが各ユーザーIDであることを確認
		assert.Equal(t, []byte(events[i].UserID), msg.Key)
	}
}
