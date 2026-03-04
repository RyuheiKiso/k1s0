package messaging

import (
	"context"
	"time"

	"github.com/google/uuid"
)

// EventMetadata 邵ｺ・ｯ郢ｧ・､郢晏生ﾎｦ郢晏現繝ｻ郢晢ｽ｡郢ｧ・ｿ郢昴・繝ｻ郢ｧ・ｿ邵ｲ繝ｻ
type EventMetadata struct {
	// EventId 邵ｺ・ｯ郢ｧ・､郢晏生ﾎｦ郢晏現繝ｻ闕ｳﾂ隲｢荳岩・髫ｴ莨懈肩陝・・ﾂ繝ｻ
	EventId string
	// EventType 邵ｺ・ｯ郢ｧ・､郢晏生ﾎｦ郢晏現繝ｻ陜吝唱骭千ｸｲ繧・ｽｾ繝ｻ "user.created.v1"
	EventType string
	// CorrelationId 邵ｺ・ｯ郢晢ｽｪ郢ｧ・ｯ郢ｧ・ｨ郢ｧ・ｹ郢晁ご蠍碁ｫ｢・｢ ID邵ｲ繝ｻ
	CorrelationId string
	// TraceId 邵ｺ・ｯ陋ｻ繝ｻ豺ｵ郢晏現ﾎ樒ｹ晢ｽｼ郢ｧ・ｷ郢晢ｽｳ郢ｧ・ｰ邵ｺ・ｮ郢晏現ﾎ樒ｹ晢ｽｼ郢ｧ・ｹ ID邵ｲ繝ｻ
	TraceId string
	// Timestamp 邵ｺ・ｯ郢ｧ・､郢晏生ﾎｦ郢晁ご蛹ｱ騾墓ｻ灘・陋ｻ・ｻ邵ｲ繝ｻ
	Timestamp time.Time
	// Source 邵ｺ・ｯ郢ｧ・､郢晏生ﾎｦ郢晉｣ｯﾂ竏ｽ・ｿ・｡陷医・縺礼ｹ晢ｽｼ郢晁侭縺幄惺髦ｪﾂ繝ｻ
	Source string
	// SchemaVersion 邵ｺ・ｯ郢ｧ・ｹ郢ｧ・ｭ郢晢ｽｼ郢晄ｧｭ繝ｰ郢晢ｽｼ郢ｧ・ｸ郢晢ｽｧ郢晢ｽｳ邵ｲ繝ｻ
	SchemaVersion int32
}

// NewEventMetadata 邵ｺ・ｯ隴・ｽｰ邵ｺ蜉ｱ・・EventMetadata 郢ｧ蝣､蜃ｽ隰瑚・笘・ｹｧ荵敖繝ｻ
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

// EventEnvelope 邵ｺ・ｯ郢ｧ・､郢晏生ﾎｦ郢晏現繝ｻ郢ｧ・ｨ郢晢ｽｳ郢晏生ﾎ溽ｹ晢ｽｼ郢晄圜・ｼ蛹ｻﾎ鍋ｹｧ・ｿ郢昴・繝ｻ郢ｧ・ｿ + 郢晏｣ｹ縺・ｹ晢ｽｭ郢晢ｽｼ郢昜ｼ夲ｽｼ蟲ｨﾂ繝ｻ
type EventEnvelope struct {
	// Metadata 邵ｺ・ｯ郢ｧ・､郢晏生ﾎｦ郢晏現繝ｻ郢晢ｽ｡郢ｧ・ｿ郢昴・繝ｻ郢ｧ・ｿ邵ｲ繝ｻ
	Metadata EventMetadata
	// Topic 邵ｺ・ｯ鬨ｾ竏ｽ・ｿ・｡陷医・Kafka 郢晏現繝ｴ郢昴・縺題惺髦ｪﾂ繝ｻ
	Topic string
	// Key 邵ｺ・ｯ郢昜ｻ｣繝ｻ郢昴・縺・ｹｧ・ｷ郢晢ｽｧ郢晢ｽｳ郢ｧ・ｭ郢晢ｽｼ繝ｻ莠包ｽｾ繝ｻ user_id繝ｻ蟲ｨﾂ繝ｻ
	Key string
	// Payload 邵ｺ・ｯ郢ｧ・､郢晏生ﾎｦ郢晏現繝ｻ郢晏｣ｹ縺・ｹ晢ｽｭ郢晢ｽｼ郢昜ｼ夲ｽｼ莠包ｽｻ・ｻ隲｢荳翫・陜吝・・ｼ蟲ｨﾂ繝ｻ
	Payload interface{}
	// Headers 邵ｺ・ｯ Kafka 郢晢ｽ｡郢昴・縺晉ｹ晢ｽｼ郢ｧ・ｸ郢晏･繝｣郢敖郢晢ｽｼ繝ｻ閧ｲ諤宣｡・･陷ｿ・ｯ髢ｭ・ｽ繝ｻ蟲ｨﾂ繝ｻ
	Headers map[string]string
}

// EventHandler 邵ｺ・ｯ郢ｧ・､郢晏生ﾎｦ郢晏現・定怎・ｦ騾・・笘・ｹｧ荵昴Ρ郢晢ｽｳ郢晏ｳｨﾎ帷ｹ晢ｽｼ鬮｢・｢隰ｨ・ｰ陜吩ｹ敖繝ｻ
type EventHandler func(ctx context.Context, event EventEnvelope) error

// EventProducer 邵ｺ・ｯ郢ｧ・､郢晏生ﾎｦ郢晏現・・Kafka 邵ｺ・ｫ鬨ｾ竏ｽ・ｿ・｡邵ｺ蜷ｶ・狗ｹｧ・､郢晢ｽｳ郢ｧ・ｿ郢晢ｽｼ郢晁ｼ斐♂郢晢ｽｼ郢ｧ・ｹ邵ｲ繝ｻ
type EventProducer interface {
	Publish(ctx context.Context, event EventEnvelope) error
	PublishBatch(ctx context.Context, events []EventEnvelope) error
	Close() error
}

// EventConsumer 邵ｺ・ｯ Kafka 邵ｺ荵晢ｽ臥ｹｧ・､郢晏生ﾎｦ郢晏現・定愾蠍ｺ・ｿ・｡邵ｺ蜷ｶ・狗ｹｧ・､郢晢ｽｳ郢ｧ・ｿ郢晢ｽｼ郢晁ｼ斐♂郢晢ｽｼ郢ｧ・ｹ邵ｲ繝ｻ
type EventConsumer interface {
	// Subscribe 邵ｺ・ｯ郢晏現繝ｴ郢昴・縺醍ｹｧ螳夲ｽｳ・ｼ髫ｱ・ｭ邵ｺ蜉ｱﾂ竏壹≧郢晏生ﾎｦ郢晏現・堤ｹ昜ｸ莞ｦ郢晏ｳｨﾎ帷ｹ晢ｽｼ邵ｺ・ｧ陷・ｽｦ騾・・笘・ｹｧ荵敖繝ｻ
	Subscribe(ctx context.Context, topic string, handler EventHandler) error
	// Close 邵ｺ・ｯ郢ｧ・ｳ郢晢ｽｳ郢ｧ・ｷ郢晢ｽ･郢晢ｽｼ郢晄ｧｭ繝ｻ郢ｧ蟶晏陶邵ｺ蛟･・狗ｸｲ繝ｻ
	Close() error
}

// MessagingError 邵ｺ・ｯ郢晢ｽ｡郢昴・縺晉ｹ晢ｽｼ郢ｧ・ｸ郢晢ｽｳ郢ｧ・ｰ隰ｫ蝣ｺ・ｽ諛翫・郢ｧ・ｨ郢晢ｽｩ郢晢ｽｼ邵ｲ繝ｻ
type MessagingError struct {
	Op  string
	Err error
}

// Error 邵ｺ・ｯ MessagingError 邵ｺ・ｮ隴√・・ｭ諤懊・髯ｦ・ｨ霑ｴ・ｾ郢ｧ螳夲ｽｿ譁絶・邵ｲ繝ｻ
func (e *MessagingError) Error() string {
	return e.Op + ": " + e.Err.Error()
}

// Unwrap 邵ｺ・ｯ陷医・繝ｻ郢ｧ・ｨ郢晢ｽｩ郢晢ｽｼ郢ｧ螳夲ｽｿ譁絶・邵ｲ繝ｻ
func (e *MessagingError) Unwrap() error {
	return e.Err
}
