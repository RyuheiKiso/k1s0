package messaging

import (
	"context"
	"time"

	"github.com/google/uuid"
)

// EventMetadata はイベントのメタデータ。
type EventMetadata struct {
	// EventId はイベントの一意識別子。
	EventId string
	// EventType はイベント種別。例: "user.created.v1"
	EventType string
	// CorrelationId はリクエスト相関 ID。
	CorrelationId string
	// TraceId は分散トレーシング用トレース ID。
	TraceId string
	// Timestamp はイベント発生日時。
	Timestamp time.Time
	// Source はイベント発行元サービス名。
	Source string
	// SchemaVersion はスキーマバージョン。
	SchemaVersion int32
}

// NewEventMetadata は新しい EventMetadata を生成する。
func NewEventMetadata(eventType, correlationId, source string) EventMetadata {
	return EventMetadata{
		EventId:       uuid.New().String(),
		EventType:     eventType,
		CorrelationId: correlationId,
		Timestamp:     time.Now().UTC(),
		Source:        source,
		SchemaVersion: 1,
	}
}

// WithTraceId sets TraceId and returns a copied metadata instance (builder-style).
func (m EventMetadata) WithTraceId(traceId string) EventMetadata {
	m.TraceId = traceId
	return m
}

// EventEnvelope はイベントのエンベロープ。メタデータ + ペイロードを包含する。
type EventEnvelope struct {
	// Metadata はイベントのメタデータ。
	Metadata EventMetadata
	// Topic は送信先の Kafka トピック名。
	Topic string
	// Key はパーティショニングキー。例: user_id 等。
	Key string
	// Payload はイベントのペイロード（シリアライズ前のデータ）。
	Payload interface{}
	// Headers は Kafka メッセージヘッダー。追跡情報等を格納する。
	Headers map[string]string
}

// EventHandler はイベント受信時のハンドラー関数インターフェース。
type EventHandler func(ctx context.Context, event EventEnvelope) error

// EventProducer はイベントを Kafka に送信するプロデューサーインターフェース。
type EventProducer interface {
	Publish(ctx context.Context, event EventEnvelope) error
	PublishBatch(ctx context.Context, events []EventEnvelope) error
	Close() error
}

// EventConsumer は Kafka からイベントを受信するコンシューマーインターフェース。
type EventConsumer interface {
	// Subscribe はトピックを購読し、イベント受信時にハンドラーを呼び出す。
	Subscribe(ctx context.Context, topic string, handler EventHandler) error
	// Close はコンシューマーを終了する。
	Close() error
}

// MessagingError はメッセージング固有のエラー。
type MessagingError struct {
	Op  string
	Err error
}

// Error は MessagingError の文字列表現を返す。
// Err が nil の場合はオペレーション名のみ返す。
func (e *MessagingError) Error() string {
	if e.Err == nil {
		return e.Op
	}
	return e.Op + ": " + e.Err.Error()
}

// Unwrap は内部エラーを返す。
func (e *MessagingError) Unwrap() error {
	return e.Err
}
