package messaging

import (
	"context"
	"encoding/json"
	"testing"

	"github.com/k1s0-platform/system-server-go-config/internal/domain/model"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestRoundTrip_ProducerConsumer はプロデューサーが送信したメッセージを
// コンシューマー側で正しくデシリアライズできることを検証する。
func TestRoundTrip_ProducerConsumer(t *testing.T) {
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

	// コンシューマー側のデシリアライズをシミュレート
	var consumed model.ConfigChangeLog
	err = json.Unmarshal(msg.Value, &consumed)
	require.NoError(t, err)

	// 全フィールドが正しく復元されることを確認
	assert.Equal(t, log.ID, consumed.ID)
	assert.Equal(t, log.ConfigEntryID, consumed.ConfigEntryID)
	assert.Equal(t, log.Namespace, consumed.Namespace)
	assert.Equal(t, log.Key, consumed.Key)
	assert.JSONEq(t, string(log.OldValue), string(consumed.OldValue))
	assert.JSONEq(t, string(log.NewValue), string(consumed.NewValue))
	assert.Equal(t, log.OldVersion, consumed.OldVersion)
	assert.Equal(t, log.NewVersion, consumed.NewVersion)
	assert.Equal(t, log.ChangeType, consumed.ChangeType)
	assert.Equal(t, log.ChangedBy, consumed.ChangedBy)
	assert.Equal(t, log.ChangedAt, consumed.ChangedAt)
}

// TestRoundTrip_MultipleEvents は複数の設定変更イベントを連続送信した場合の
// デシリアライズを検証する。
func TestRoundTrip_MultipleEvents(t *testing.T) {
	mock := &mockWriter{}
	p := &KafkaProducer{
		writer: mock,
		topic:  "k1s0.system.config.changed.v1",
	}

	oldVal1, _ := json.Marshal(10)
	newVal1, _ := json.Marshal(20)
	newVal2, _ := json.Marshal("new_issuer")

	events := []*model.ConfigChangeLog{
		{
			ID:            "id-1",
			ConfigEntryID: "entry-1",
			Namespace:     "system.auth.database",
			Key:           "max_connections",
			OldValue:      oldVal1,
			NewValue:      newVal1,
			OldVersion:    1,
			NewVersion:    2,
			ChangeType:    "UPDATED",
			ChangedBy:     "admin@example.com",
		},
		{
			ID:            "id-2",
			ConfigEntryID: "entry-2",
			Namespace:     "system.auth.jwt",
			Key:           "issuer",
			OldValue:      nil,
			NewValue:      newVal2,
			OldVersion:    0,
			NewVersion:    1,
			ChangeType:    "CREATED",
			ChangedBy:     "admin@example.com",
		},
	}

	for _, e := range events {
		err := p.Publish(context.Background(), e)
		require.NoError(t, err)
	}

	require.Len(t, mock.messages, 2)

	for i, msg := range mock.messages {
		var consumed model.ConfigChangeLog
		err := json.Unmarshal(msg.Value, &consumed)
		require.NoError(t, err)
		assert.Equal(t, events[i].ID, consumed.ID)
		assert.Equal(t, events[i].Namespace, consumed.Namespace)
		assert.Equal(t, events[i].Key, consumed.Key)
		assert.Equal(t, events[i].ChangeType, consumed.ChangeType)
		// パーティションキーが namespace.key であることを確認
		expectedKey := events[i].Namespace + "." + events[i].Key
		assert.Equal(t, []byte(expectedKey), msg.Key)
	}
}
