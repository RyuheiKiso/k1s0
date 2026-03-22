package messaging_test

import (
	"context"
	"errors"
	"testing"

	messaging "github.com/k1s0-platform/system-library-go-messaging"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// NewEventMetadataがイベントメタデータを正しく生成することを確認する。
func TestNewEventMetadata(t *testing.T) {
	meta := messaging.NewEventMetadata("user.created.v1", "corr-123", "auth-service")
	assert.NotEmpty(t, meta.EventId)
	assert.Equal(t, "user.created.v1", meta.EventType)
	assert.Equal(t, "corr-123", meta.CorrelationId)
	assert.Equal(t, "auth-service", meta.Source)
	assert.Equal(t, int32(1), meta.SchemaVersion)
	assert.False(t, meta.Timestamp.IsZero())
}

// NewEventMetadataが呼び出しごとに一意なイベントIDを生成することを確認する。
func TestNewEventMetadata_UniqueIds(t *testing.T) {
	meta1 := messaging.NewEventMetadata("event.v1", "corr-1", "svc")
	meta2 := messaging.NewEventMetadata("event.v1", "corr-1", "svc")
	assert.NotEqual(t, meta1.EventId, meta2.EventId)
}

// NoOpEventProducerのPublishがエラーなしでイベントを受け付け、カウントを増加することを確認する。
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

// NoOpEventProducerが複数回のPublishを正確にカウントすることを確認する。
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

// NoOpEventProducerのPublishBatchが複数のイベントをまとめて受け付けることを確認する。
func TestNoOpEventProducer_PublishBatch(t *testing.T) {
	producer := &messaging.NoOpEventProducer{}
	events := []messaging.EventEnvelope{
		{
			Metadata: messaging.NewEventMetadata("test.v1", "corr-1", "svc"),
			Topic:    "k1s0.system.test.event.v1",
		},
		{
			Metadata: messaging.NewEventMetadata("test.v1", "corr-2", "svc"),
			Topic:    "k1s0.system.test.event.v1",
		},
	}
	err := producer.PublishBatch(context.Background(), events)
	require.NoError(t, err)
	assert.Equal(t, 2, producer.PublishedCount())
}

// NoOpEventProducerにErrが設定されている場合、Publishがそのエラーを返すことを確認する。
func TestNoOpEventProducer_Publish_WithError(t *testing.T) {
	expectedErr := errors.New("publish failed")
	producer := &messaging.NoOpEventProducer{Err: expectedErr}
	err := producer.Publish(context.Background(), messaging.EventEnvelope{})
	assert.ErrorIs(t, err, expectedErr)
	assert.Equal(t, 0, producer.PublishedCount())
}

// NoOpEventProducerのCloseがプロデューサーをクローズ済み状態にすることを確認する。
func TestNoOpEventProducer_Close(t *testing.T) {
	producer := &messaging.NoOpEventProducer{}
	assert.False(t, producer.IsClosed())
	err := producer.Close()
	require.NoError(t, err)
	assert.True(t, producer.IsClosed())
}

// MessagingErrorのエラーメッセージにオペレーション名と原因が含まれることを確認する。
func TestMessagingError(t *testing.T) {
	cause := errors.New("connection refused")
	err := &messaging.MessagingError{Op: "Publish", Err: cause}
	assert.Contains(t, err.Error(), "Publish")
	assert.Contains(t, err.Error(), "connection refused")
	assert.ErrorIs(t, err, cause)
}

// EventEnvelopeにHeadersフィールドを設定して正しく取得できることを確認する。
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

// NoOpEventProducerがPublishしたイベントをPublishedスライスに記録することを確認する。
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

// NoOpEventProducerがEventProducerインターフェースを実装していることをコンパイル時に確認する。
func TestEventProducer_InterfaceCompliance(t *testing.T) {
	// NoOpEventProducer 縺ｯ EventProducer 繧､繝ｳ繧ｿ繝ｼ繝輔ぉ繝ｼ繧ｹ繧貞ｮ溯｣・＠縺ｦ縺・ｋ縺薙→繧堤｢ｺ隱・
	var _ messaging.EventProducer = &messaging.NoOpEventProducer{}
}

// NewEventMetadataで生成されたメタデータにTraceIdフィールドが存在することを確認する。
func TestNewEventMetadata_HasTraceId(t *testing.T) {
	meta := messaging.NewEventMetadata("user.created.v1", "corr-001", "auth-service")
	// TraceId 縺ｯ繝・ヵ繧ｩ繝ｫ繝育ｩｺ・亥､夜Κ縺九ｉ險ｭ螳夲ｼ・
	_ = meta.TraceId // 繝輔ぅ繝ｼ繝ｫ繝峨′蟄伜惠縺吶ｋ縺薙→繧堤｢ｺ隱・
}

// WithTraceIdが元のメタデータを変更せず新しいメタデータにトレースIDを設定することを確認する。
func TestEventMetadata_WithTraceId(t *testing.T) {
	meta := messaging.NewEventMetadata("user.created.v1", "corr-001", "auth-service")
	withTrace := meta.WithTraceId("trace-abc-123")

	assert.Equal(t, "trace-abc-123", withTrace.TraceId)
	assert.Empty(t, meta.TraceId)
}

// EventEnvelopeの全フィールド（Topic、Payload、Headers）が正しく設定されることを確認する。
func TestEventEnvelope_Fields(t *testing.T) {
	meta := messaging.NewEventMetadata("task.created.v1", "corr-002", "task-server")
	envelope := messaging.EventEnvelope{
		Metadata: meta,
		Topic:    "k1s0.business.task.created.v1",
		Payload:  map[string]int{"order_id": 42},
		Headers:  map[string]string{"version": "1"},
	}
	assert.Equal(t, "k1s0.business.task.created.v1", envelope.Topic)
	assert.Equal(t, "1", envelope.Headers["version"])
	assert.Equal(t, map[string]int{"order_id": 42}, envelope.Payload)
}

// NoOpEventProducerがClose後もPublishを受け付けエラーを返さないことを確認する。
func TestNoOpEventProducer_PublishAfterClose(t *testing.T) {
	producer := &messaging.NoOpEventProducer{}
	require.NoError(t, producer.Close())
	assert.True(t, producer.IsClosed())
	// Close 蠕後ｂ Publish 縺ｯ蜻ｼ縺ｹ繧具ｼ医お繝ｩ繝ｼ縺励↑縺・ｼ・
	err := producer.Publish(context.Background(), messaging.EventEnvelope{})
	require.NoError(t, err)
}

// MessagingErrorのエラーラップが正しく機能し、ErrorIsによる原因判定ができることを確認する。
func TestMessagingError_IsNilSafe(t *testing.T) {
	cause := errors.New("timeout")
	err := &messaging.MessagingError{Op: "Subscribe", Err: cause}
	assert.ErrorIs(t, err, cause)
	assert.Contains(t, err.Error(), "Subscribe")
	assert.Contains(t, err.Error(), "timeout")
}

// MessagingErrorのErrがnilでもError()がパニックしないことを確認する。
func TestMessagingError_NilErr(t *testing.T) {
	err := &messaging.MessagingError{Op: "Publish", Err: nil}
	// Err が nil の場合はオペレーション名のみ返し、パニックしないことを確認する
	assert.Equal(t, "Publish", err.Error())
}

// EventEnvelopeのHeadersおよびPayloadがnilでも正常に扱えることを確認する。
func TestEventEnvelope_NilHeadersAllowed(t *testing.T) {
	meta := messaging.NewEventMetadata("test.v1", "corr-003", "svc")
	envelope := messaging.EventEnvelope{
		Metadata: meta,
		Topic:    "test.topic",
	}
	assert.Nil(t, envelope.Headers)
	assert.Nil(t, envelope.Payload)
}
