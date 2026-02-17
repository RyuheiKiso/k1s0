package messaging

import (
	"context"
	"encoding/json"
	"fmt"

	kafka "github.com/segmentio/kafka-go"

	"github.com/k1s0-platform/system-server-go-auth/internal/domain/model"
	"github.com/k1s0-platform/system-server-go-auth/internal/infra/config"
)

// messageWriter は Kafka Writer の抽象インターフェース。
// テスト時にモックへ差し替え可能にする。
type messageWriter interface {
	WriteMessages(ctx context.Context, msgs ...writerMessage) error
	Close() error
}

// writerMessage は Kafka に送信するメッセージを表す。
type writerMessage struct {
	Topic string
	Key   []byte
	Value []byte
}

// kafkaGoWriter は kafka-go の Writer をラップする本番実装。
type kafkaGoWriter struct {
	w *kafka.Writer
}

func (k *kafkaGoWriter) WriteMessages(ctx context.Context, msgs ...writerMessage) error {
	kafkaMsgs := make([]kafka.Message, len(msgs))
	for i, m := range msgs {
		kafkaMsgs[i] = kafka.Message{
			Topic: m.Topic,
			Key:   m.Key,
			Value: m.Value,
		}
	}
	return k.w.WriteMessages(ctx, kafkaMsgs...)
}

func (k *kafkaGoWriter) Close() error {
	return k.w.Close()
}

// KafkaProducer は Kafka プロデューサー。
type KafkaProducer struct {
	writer messageWriter
	topic  string
}

// NewKafkaProducer は新しい KafkaProducer を作成する。
func NewKafkaProducer(cfg config.KafkaConfig) *KafkaProducer {
	w := &kafka.Writer{
		Addr:         kafka.TCP(cfg.Brokers...),
		Balancer:     &kafka.Hash{},       // パーティションキーによる分散
		RequiredAcks: kafka.RequireAll,     // acks=all
		Async:        false,
	}
	return &KafkaProducer{
		writer: &kafkaGoWriter{w: w},
		topic:  cfg.Topics.Publish,
	}
}

// Publish は監査ログイベントを Kafka に配信する。
func (p *KafkaProducer) Publish(ctx context.Context, log *model.AuditLog) error {
	data, err := json.Marshal(log)
	if err != nil {
		return fmt.Errorf("failed to serialize audit log: %w", err)
	}

	msg := writerMessage{
		Topic: p.topic,
		Key:   []byte(log.UserID),
		Value: data,
	}

	if err := p.writer.WriteMessages(ctx, msg); err != nil {
		return fmt.Errorf("failed to publish audit event: %w", err)
	}

	return nil
}

// Close は Kafka プロデューサーを閉じる。
func (p *KafkaProducer) Close() error {
	return p.writer.Close()
}
