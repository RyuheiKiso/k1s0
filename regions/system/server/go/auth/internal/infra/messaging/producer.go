package messaging

import (
	"context"

	"github.com/k1s0-platform/system-server-go-auth/internal/domain/model"
	"github.com/k1s0-platform/system-server-go-auth/internal/infra/config"
)

// KafkaProducer は Kafka プロデューサー。
type KafkaProducer struct {
	brokers []string
	topic   string
}

// NewKafkaProducer は新しい KafkaProducer を作成する。
func NewKafkaProducer(cfg config.KafkaConfig) *KafkaProducer {
	return &KafkaProducer{
		brokers: cfg.Brokers,
		topic:   cfg.Topic,
	}
}

// Publish は監査ログイベントを Kafka に配信する。
// 本番実装では kafka-go Writer を使う。
func (p *KafkaProducer) Publish(ctx context.Context, log *model.AuditLog) error {
	// 本番実装:
	// 1. AuditLog を JSON にシリアライズ
	// 2. Kafka トピックに配信
	return nil
}

// Close は Kafka プロデューサーを閉じる。
func (p *KafkaProducer) Close() error {
	return nil
}
