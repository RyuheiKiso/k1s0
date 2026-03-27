package messaging

import (
	"context"
	"time"

	"github.com/google/uuid"
)

// EventMetadata はイベントのメタデータ。
type EventMetadata struct {
	// EventID はイベントの一意識別子（L-3 監査対応: EventId → EventID に改名）。
	EventID string
	// EventType はイベント種別。例: "user.created.v1"
	EventType string
	// CorrelationID はリクエスト相関 ID（L-3 監査対応: CorrelationId → CorrelationID に改名）。
	CorrelationID string
	// TraceID は分散トレーシング用トレース ID（L-3 監査対応: TraceId → TraceID に改名）。
	TraceID string
	// Timestamp はイベント発生日時。
	Timestamp time.Time
	// Source はイベント発行元サービス名。
	Source string
	// SchemaVersion はスキーマバージョン。
	SchemaVersion int32
}

// NewEventMetadata は新しい EventMetadata を生成する。
func NewEventMetadata(eventType, correlationID, source string) EventMetadata {
	return EventMetadata{
		EventID:       uuid.New().String(),
		EventType:     eventType,
		CorrelationID: correlationID,
		Timestamp:     time.Now().UTC(),
		Source:        source,
		SchemaVersion: 1,
	}
}

// WithTraceID はトレース ID を設定して EventMetadata のコピーを返す（builder スタイル）。
// L-3 監査対応: WithTraceId → WithTraceID に改名。
func (m EventMetadata) WithTraceID(traceID string) EventMetadata {
	m.TraceID = traceID
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
	// Payload はイベントのペイロード（シリアライズ前のデータ、interface{} → any: Go 1.18+ 推奨エイリアスを使用する）。
	Payload any
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

// Err は MessagingError の短縮エイリアス（L-3 監査対応: stutter 命名解消）。
// 注意: builtin error との混同を避けるため Err を使用する。
// 新しいコードでは messaging.Err を使用すること。
type Err = MessagingError
