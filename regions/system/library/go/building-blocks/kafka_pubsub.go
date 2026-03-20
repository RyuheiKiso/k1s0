package buildingblocks

import (
	"context"
	"log/slog"
	"sync"
	"time"
)

// KafkaEventEnvelope はk1s0-messaging の EventEnvelope と互換性を持つ最小限のイベントエンベロープ。
// Kafka メッセージの内容を表すデータ構造として使用する。
type KafkaEventEnvelope struct {
	// Topic はメッセージの送受信先となる Kafka トピック名。
	Topic string
	// Key はメッセージキー（パーティショニングや Message ID として使用）。
	Key string
	// Payload はメッセージの本文データ（バイト列）。
	Payload []byte
	// Headers はメッセージに付随するカスタムヘッダーのマップ。
	Headers map[string]string
}

// KafkaEventProducer はk1s0-messaging の EventProducer と互換性を持つプロデューサーインターフェース。
// *kafka.Producer を注入することでこのインターフェースを満たせる。
type KafkaEventProducer interface {
	// Publish は指定したイベントエンベロープを Kafka に送信する。
	Publish(ctx context.Context, event KafkaEventEnvelope) error
	// Close はプロデューサーの接続を閉じてリソースを解放する。
	Close() error
}

// KafkaEventHandler は受信した各イベントに対して呼び出されるコールバック関数の型。
type KafkaEventHandler func(ctx context.Context, event KafkaEventEnvelope) error

// KafkaEventConsumer はk1s0-messaging の EventConsumer と互換性を持つコンシューマーインターフェース。
// *kafka.Consumer を注入することでこのインターフェースを満たせる。
type KafkaEventConsumer interface {
	// Subscribe は指定トピックの購読を開始し、受信イベントをハンドラーに渡す。
	Subscribe(ctx context.Context, topic string, handler KafkaEventHandler) error
	// Close はコンシューマーの接続を閉じてリソースを解放する。
	Close() error
}

// コンパイル時にインターフェース準拠を検証する。
var _ PubSub = (*KafkaPubSub)(nil)

// KafkaPubSub は Kafka をバックエンドとする PubSub 実装。
// KafkaEventProducer でメッセージを発行し、オプションの KafkaEventConsumer でメッセージを購読する。
type KafkaPubSub struct {
	// mu は status フィールドへの並行アクセスを保護する読み書きミューテックス。
	mu sync.RWMutex
	// name はコンポーネントの識別子。
	name string
	// producer はKafkaへのメッセージ発行を担うプロデューサー実装。
	producer KafkaEventProducer
	// consumer はKafkaからのメッセージ購読を担うコンシューマー実装（nil許容）。
	consumer KafkaEventConsumer
	// status はコンポーネントの現在の状態を表す。
	status ComponentStatus
}

// NewKafkaPubSub は新しい KafkaPubSub を生成して返す。
// consumer は発行のみ行う場合は nil を渡してよい。
func NewKafkaPubSub(name string, producer KafkaEventProducer, consumer KafkaEventConsumer) *KafkaPubSub {
	return &KafkaPubSub{
		name:     name,
		producer: producer,
		consumer: consumer,
		status:   StatusUninitialized,
	}
}

// Name はコンポーネント識別子を返す。
func (p *KafkaPubSub) Name() string { return p.name }

// Version はコンポーネントのバージョン文字列を返す。
func (p *KafkaPubSub) Version() string { return "1.0.0" }

// Init はコンポーネントを初期化し、ステータスを Ready に遷移させる。
func (p *KafkaPubSub) Init(_ context.Context, _ Metadata) error {
	p.mu.Lock()
	defer p.mu.Unlock()
	p.status = StatusReady
	return nil
}

// Close はプロデューサーおよびコンシューマーを閉じ、ステータスを Closed に遷移させる。
// コンシューマーが nil の場合はプロデューサーのみを閉じる。
func (p *KafkaPubSub) Close(_ context.Context) error {
	p.mu.Lock()
	defer p.mu.Unlock()
	if err := p.producer.Close(); err != nil {
		return NewComponentError(p.name, "Close", "failed to close Kafka producer", err)
	}
	// コンシューマーが設定されている場合のみ閉じる。
	if p.consumer != nil {
		if err := p.consumer.Close(); err != nil {
			return NewComponentError(p.name, "Close", "failed to close Kafka consumer", err)
		}
	}
	p.status = StatusClosed
	return nil
}

// Status はコンポーネントの現在のステータスを返す。
func (p *KafkaPubSub) Status(_ context.Context) ComponentStatus {
	p.mu.RLock()
	defer p.mu.RUnlock()
	return p.status
}

// Publish は指定した Kafka トピックへメッセージを送信する。
// Message の ID をKafkaメッセージキーとして使用し、Data をペイロードとして送る。
func (p *KafkaPubSub) Publish(ctx context.Context, msg *Message) error {
	// Message を KafkaEventEnvelope に変換してプロデューサーに渡す。
	env := KafkaEventEnvelope{
		Topic:   msg.Topic,
		Key:     msg.ID,
		Payload: msg.Data,
		Headers: msg.Metadata,
	}
	if err := p.producer.Publish(ctx, env); err != nil {
		return NewComponentError(p.name, "Publish", "failed to publish to Kafka", err)
	}
	return nil
}

// Subscribe は指定した Kafka トピックのメッセージを受信するチャネルを返す。
// コンシューマーが nil の場合はエラーを返す。
// チャネルのバッファサイズは64で、バッファが満杯の場合はメッセージをドロップする。
func (p *KafkaPubSub) Subscribe(ctx context.Context, topic string) (<-chan *Message, error) {
	if p.consumer == nil {
		return nil, NewComponentError(p.name, "Subscribe", "consumer is not configured", nil)
	}
	// バッファ付きチャネルを作成し、受信側の処理が遅れてもブロックを防ぐ。
	ch := make(chan *Message, 64)
	handler := func(ctx context.Context, env KafkaEventEnvelope) error {
		// KafkaEventEnvelope を Message に変換してチャネルへ送信する。
		msg := &Message{
			Topic:     env.Topic,
			Data:      env.Payload,
			Metadata:  env.Headers,
			ID:        env.Key,
			Timestamp: time.Now(),
		}
		// チャネルが満杯の場合はメッセージをドロップして処理を継続する。
		select {
		case ch <- msg:
		default:
			// バッファが満杯のためメッセージをドロップした場合は警告ログを出力する
			slog.Warn("Kafka メッセージドロップ: 受信バッファが満杯",
				slog.String("topic", env.Topic),
				slog.String("key", env.Key),
			)
		}
		return nil
	}
	if err := p.consumer.Subscribe(ctx, topic, handler); err != nil {
		return nil, NewComponentError(p.name, "Subscribe", "failed to subscribe to Kafka topic", err)
	}
	// コンテキストがキャンセルされたときにチャネルをクローズして購読者にEOFを通知する
	go func() {
		<-ctx.Done()
		close(ch)
	}()
	return ch, nil
}
