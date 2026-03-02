package messaging

import (
	"context"
	"time"

	"github.com/google/uuid"
)

// EventMetadata 縺ｯ繧､繝吶Φ繝医・繝｡繧ｿ繝・・繧ｿ縲・
type EventMetadata struct {
	// EventId 縺ｯ繧､繝吶Φ繝医・荳諢上↑隴伜挨蟄舌・
	EventId string
	// EventType 縺ｯ繧､繝吶Φ繝医・蝙句錐縲ゆｾ・ "user.created.v1"
	EventType string
	// CorrelationId 縺ｯ繝ｪ繧ｯ繧ｨ繧ｹ繝育嶌髢｢ ID縲・
	CorrelationId string
	// TraceId 縺ｯ蛻・淵繝医Ξ繝ｼ繧ｷ繝ｳ繧ｰ縺ｮ繝医Ξ繝ｼ繧ｹ ID縲・
	TraceId string
	// Timestamp 縺ｯ繧､繝吶Φ繝育匱逕滓凾蛻ｻ縲・
	Timestamp time.Time
	// Source 縺ｯ繧､繝吶Φ繝磯∽ｿ｡蜈・し繝ｼ繝薙せ蜷阪・
	Source string
	// SchemaVersion 縺ｯ繧ｹ繧ｭ繝ｼ繝槭ヰ繝ｼ繧ｸ繝ｧ繝ｳ縲・
	SchemaVersion int32
}

// NewEventMetadata 縺ｯ譁ｰ縺励＞ EventMetadata 繧堤函謌舌☆繧九・
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

// EventEnvelope 縺ｯ繧､繝吶Φ繝医・繧ｨ繝ｳ繝吶Ο繝ｼ繝暦ｼ医Γ繧ｿ繝・・繧ｿ + 繝壹う繝ｭ繝ｼ繝会ｼ峨・
type EventEnvelope struct {
	// Metadata 縺ｯ繧､繝吶Φ繝医・繝｡繧ｿ繝・・繧ｿ縲・
	Metadata EventMetadata
	// Topic 縺ｯ騾∽ｿ｡蜈・Kafka 繝医ヴ繝・け蜷阪・
	Topic string
	// Key 縺ｯ繝代・繝・ぅ繧ｷ繝ｧ繝ｳ繧ｭ繝ｼ・井ｾ・ user_id・峨・
	Key string
	// Payload 縺ｯ繧､繝吶Φ繝医・繝壹う繝ｭ繝ｼ繝会ｼ井ｻｻ諢上・蝙具ｼ峨・
	Payload interface{}
	// Headers 縺ｯ Kafka 繝｡繝・そ繝ｼ繧ｸ繝倥ャ繝繝ｼ・育怐逡･蜿ｯ閭ｽ・峨・
	Headers map[string]string
}

// EventHandler 縺ｯ繧､繝吶Φ繝医ｒ蜃ｦ逅・☆繧九ワ繝ｳ繝峨Λ繝ｼ髢｢謨ｰ蝙九・
type EventHandler func(ctx context.Context, event EventEnvelope) error

// EventProducer 縺ｯ繧､繝吶Φ繝医ｒ Kafka 縺ｫ騾∽ｿ｡縺吶ｋ繧､繝ｳ繧ｿ繝ｼ繝輔ぉ繝ｼ繧ｹ縲・
type EventProducer interface {
	Publish(ctx context.Context, event EventEnvelope) error
	PublishBatch(ctx context.Context, events []EventEnvelope) error
	Close() error
}

// EventConsumer 縺ｯ Kafka 縺九ｉ繧､繝吶Φ繝医ｒ蜿嶺ｿ｡縺吶ｋ繧､繝ｳ繧ｿ繝ｼ繝輔ぉ繝ｼ繧ｹ縲・
type EventConsumer interface {
	// Subscribe 縺ｯ繝医ヴ繝・け繧定ｳｼ隱ｭ縺励√う繝吶Φ繝医ｒ繝上Φ繝峨Λ繝ｼ縺ｧ蜃ｦ逅・☆繧九・
	Subscribe(ctx context.Context, topic string, handler EventHandler) error
	// Close 縺ｯ繧ｳ繝ｳ繧ｷ繝･繝ｼ繝槭・繧帝哩縺倥ｋ縲・
	Close() error
}

// MessagingError 縺ｯ繝｡繝・そ繝ｼ繧ｸ繝ｳ繧ｰ謫堺ｽ懊・繧ｨ繝ｩ繝ｼ縲・
type MessagingError struct {
	Op  string
	Err error
}

// Error 縺ｯ MessagingError 縺ｮ譁・ｭ怜・陦ｨ迴ｾ繧定ｿ斐☆縲・
func (e *MessagingError) Error() string {
	return e.Op + ": " + e.Err.Error()
}

// Unwrap 縺ｯ蜈・・繧ｨ繝ｩ繝ｼ繧定ｿ斐☆縲・
func (e *MessagingError) Unwrap() error {
	return e.Err
}

