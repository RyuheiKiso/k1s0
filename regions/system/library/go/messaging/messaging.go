package messaging

import (
	"context"
	"time"

	"github.com/google/uuid"
)

// EventMetadata はイベントのメタデータ。
type EventMetadata struct {
	// EventId はイベントの一意な識別子。
	EventId string
	// EventType はイベントの型名。例: "user.created.v1"
	EventType string
	// CorrelationId はリクエスト相関 ID。
	CorrelationId string
	// TraceId は分散トレーシングのトレース ID。
	TraceId string
	// Timestamp はイベント発生時刻。
	Timestamp time.Time
	// Source はイベント送信元サービス名。
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

// EventEnvelope はイベントのエンベロープ（メタデータ + ペイロード）。
type EventEnvelope struct {
	// Metadata はイベントのメタデータ。
	Metadata EventMetadata
	// Topic は送信先 Kafka トピック名。
	Topic string
	// Key はパーティションキー（例: user_id）。
	Key string
	// Payload はイベントのペイロード（任意の型）。
	Payload interface{}
	// Headers は Kafka メッセージヘッダー（省略可能）。
	Headers map[string]string
}

// EventHandler はイベントを処理するハンドラー関数型。
type EventHandler func(ctx context.Context, event EventEnvelope) error

// EventProducer はイベントを Kafka に送信するインターフェース。
type EventProducer interface {
	// Publish はイベントを Kafka トピックに送信する。
	Publish(ctx context.Context, event EventEnvelope) error
	// Close はプロデューサーを閉じる。
	Close() error
}

// EventConsumer は Kafka からイベントを受信するインターフェース。
type EventConsumer interface {
	// Subscribe はトピックを購読し、イベントをハンドラーで処理する。
	Subscribe(ctx context.Context, topic string, handler EventHandler) error
	// Close はコンシューマーを閉じる。
	Close() error
}

// MessagingError はメッセージング操作のエラー。
type MessagingError struct {
	Op  string
	Err error
}

// Error は MessagingError の文字列表現を返す。
func (e *MessagingError) Error() string {
	return e.Op + ": " + e.Err.Error()
}

// Unwrap は元のエラーを返す。
func (e *MessagingError) Unwrap() error {
	return e.Err
}
