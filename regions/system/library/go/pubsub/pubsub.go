// pubsub パッケージは Pub/Sub メッセージングのコアインターフェースと実装を提供する。
// InMemoryPubSub（テスト用）、RedisPubSub、KafkaPubSub の3種類の実装を含む。
package pubsub

import (
	"context"
	"fmt"
	"sync"
	"time"
)

// ComponentStatus はコンポーネントの現在の状態を表す文字列型。
type ComponentStatus string

// コンポーネントの状態定数。
const (
	StatusUninitialized ComponentStatus = "uninitialized"
	StatusReady         ComponentStatus = "ready"
	StatusDegraded      ComponentStatus = "degraded"
	StatusClosed        ComponentStatus = "closed"
	StatusError         ComponentStatus = "error"
)

// Metadata はコンポーネントのメタデータを保持する構造体。
type Metadata struct {
	Name    string            `json:"name"`
	Version string            `json:"version"`
	Tags    map[string]string `json:"tags,omitempty"`
}

// Component はすべてのビルディングブロックコンポーネントの基底インターフェース。
type Component interface {
	Name() string
	Version() string
	Init(ctx context.Context, metadata Metadata) error
	Close(ctx context.Context) error
	Status(ctx context.Context) ComponentStatus
}

// ComponentError はビルディングブロック操作から発生するエラーを表す。
type ComponentError struct {
	Component string
	Operation string
	Message   string
	Err       error
}

// Error はエラーメッセージを文字列として返す。
func (e *ComponentError) Error() string {
	if e.Err != nil {
		return fmt.Sprintf("[%s] %s: %s: %v", e.Component, e.Operation, e.Message, e.Err)
	}
	return fmt.Sprintf("[%s] %s: %s", e.Component, e.Operation, e.Message)
}

// Unwrap はラップされた元のエラーを返す。
func (e *ComponentError) Unwrap() error {
	return e.Err
}

// NewComponentError は新しい ComponentError を生成して返す。
func NewComponentError(component, operation, message string, err error) *ComponentError {
	return &ComponentError{Component: component, Operation: operation, Message: message, Err: err}
}

// Message は Pub/Sub のメッセージを表す構造体。
type Message struct {
	Topic     string            `json:"topic"`
	Data      []byte            `json:"data"`
	Metadata  map[string]string `json:"metadata,omitempty"`
	ID        string            `json:"id"`
	Timestamp time.Time         `json:"timestamp"`
}

// PubSub はパブリッシュ/サブスクライブメッセージング機能を提供するインターフェース。
type PubSub interface {
	Component
	Publish(ctx context.Context, msg *Message) error
	Subscribe(ctx context.Context, topic string) (<-chan *Message, error)
}

// ============================================================
// InMemoryPubSub
// ============================================================

// コンパイル時にインターフェース準拠を検証する。
var _ PubSub = (*InMemoryPubSub)(nil)

// InMemoryPubSub はテスト用のインメモリ PubSub 実装。
type InMemoryPubSub struct {
	mu     sync.RWMutex
	subs   map[string][]chan *Message
	status ComponentStatus
}

// NewInMemoryPubSub は新しい InMemoryPubSub を生成して返す。
func NewInMemoryPubSub() *InMemoryPubSub {
	return &InMemoryPubSub{
		subs:   make(map[string][]chan *Message),
		status: StatusUninitialized,
	}
}

// Name はコンポーネント識別子を返す。
func (p *InMemoryPubSub) Name() string { return "inmemory-pubsub" }

// Version はコンポーネントのバージョン文字列を返す。
func (p *InMemoryPubSub) Version() string { return "1.0.0" }

// Init はコンポーネントを初期化し、ステータスを Ready に遷移させる。
func (p *InMemoryPubSub) Init(_ context.Context, _ Metadata) error {
	p.mu.Lock()
	defer p.mu.Unlock()
	p.status = StatusReady
	return nil
}

// Close はすべてのサブスクライバーチャネルを閉じ、ステータスを Closed に遷移させる。
func (p *InMemoryPubSub) Close(_ context.Context) error {
	p.mu.Lock()
	defer p.mu.Unlock()
	for _, chans := range p.subs {
		for _, ch := range chans {
			close(ch)
		}
	}
	p.subs = make(map[string][]chan *Message)
	p.status = StatusClosed
	return nil
}

// Status はコンポーネントの現在のステータスを返す。
func (p *InMemoryPubSub) Status(_ context.Context) ComponentStatus {
	p.mu.RLock()
	defer p.mu.RUnlock()
	return p.status
}

// Publish は msg.Topic のすべてのサブスクライバーへメッセージを送信する（ノンブロッキング）。
// msg.Timestamp がゼロ値の場合は現在時刻を設定する。
func (p *InMemoryPubSub) Publish(_ context.Context, msg *Message) error {
	out := *msg
	if out.Timestamp.IsZero() {
		out.Timestamp = time.Now()
	}
	p.mu.RLock()
	defer p.mu.RUnlock()
	for _, ch := range p.subs[out.Topic] {
		select {
		case ch <- &out:
		default:
		}
	}
	return nil
}

// Subscribe は指定トピックのメッセージを受信するチャネルを返す。
func (p *InMemoryPubSub) Subscribe(_ context.Context, topic string) (<-chan *Message, error) {
	ch := make(chan *Message, 64)
	p.mu.Lock()
	defer p.mu.Unlock()
	p.subs[topic] = append(p.subs[topic], ch)
	return ch, nil
}

// ============================================================
// RedisPubSub
// ============================================================

// RedisPubSubClient は Redis の Pub/Sub 操作を担うインターフェース。
// k1s0-cache の PubSubClient と互換性を持つ。
// *cache.RedisPubSubClient を注入することでこのインターフェースを満たせる。
type RedisPubSubClient interface {
	// Publish は指定トピックへメッセージを発行する。
	Publish(ctx context.Context, topic string, payload []byte) error
	// Subscribe は指定トピックを購読し、受信ごとにハンドラーを呼び出す。
	Subscribe(ctx context.Context, topic string, handler func(ctx context.Context, payload []byte) error) error
	// Close は接続を閉じてリソースを解放する。
	Close() error
}

// コンパイル時にインターフェース準拠を検証する。
var _ PubSub = (*RedisPubSub)(nil)

// RedisPubSub は Redis をバックエンドとする PubSub 実装。
// RedisPubSubClient をラップして PubSub インターフェースを提供する。
// チャネルのバッファサイズは64で、バッファが満杯の場合はメッセージをドロップする。
type RedisPubSub struct {
	// mu は status フィールドへの並行アクセスを保護する読み書きミューテックス。
	mu sync.RWMutex
	// name はコンポーネントの識別子。
	name string
	// client は Redis Pub/Sub 操作を担うクライアント実装。
	client RedisPubSubClient
	// status はコンポーネントの現在の状態を表す。
	status ComponentStatus
}

// NewRedisPubSub は新しい RedisPubSub を生成して返す。
// name はコンポーネント識別子、client は Redis Pub/Sub 操作を担うクライアント実装。
func NewRedisPubSub(name string, client RedisPubSubClient) *RedisPubSub {
	return &RedisPubSub{
		name:   name,
		client: client,
		status: StatusUninitialized,
	}
}

// Name はコンポーネント識別子を返す。
func (p *RedisPubSub) Name() string { return p.name }

// Version はコンポーネントのバージョン文字列を返す。
func (p *RedisPubSub) Version() string { return "1.0.0" }

// Init はコンポーネントを初期化し、ステータスを Ready に遷移させる。
func (p *RedisPubSub) Init(_ context.Context, _ Metadata) error {
	p.mu.Lock()
	defer p.mu.Unlock()
	p.status = StatusReady
	return nil
}

// Close はクライアントを閉じ、ステータスを Closed に遷移させる。
func (p *RedisPubSub) Close(_ context.Context) error {
	p.mu.Lock()
	defer p.mu.Unlock()
	if err := p.client.Close(); err != nil {
		return NewComponentError(p.name, "Close", "failed to close Redis pub/sub client", err)
	}
	p.status = StatusClosed
	return nil
}

// Status はコンポーネントの現在のステータスを返す。
func (p *RedisPubSub) Status(_ context.Context) ComponentStatus {
	p.mu.RLock()
	defer p.mu.RUnlock()
	return p.status
}

// Publish は指定した Redis トピックへメッセージを送信する。
// Message.Data をペイロードとして使用する。
// Redis Pub/Sub は Kafka のようなメッセージヘッダーをサポートしないため、Metadata は送信しない。
func (p *RedisPubSub) Publish(ctx context.Context, msg *Message) error {
	if err := p.client.Publish(ctx, msg.Topic, msg.Data); err != nil {
		return NewComponentError(p.name, "Publish", "failed to publish to Redis", err)
	}
	return nil
}

// Subscribe は指定した Redis トピックのメッセージを受信するチャネルを返す。
// チャネルのバッファサイズは64で、バッファが満杯の場合はメッセージをドロップする。
func (p *RedisPubSub) Subscribe(ctx context.Context, topic string) (<-chan *Message, error) {
	// バッファ付きチャネルを作成し、受信側の処理が遅れてもブロックを防ぐ。
	ch := make(chan *Message, 64)
	handler := func(ctx context.Context, payload []byte) error {
		// 受信したペイロードを Message に変換してチャネルへ送信する。
		msg := &Message{
			Topic:     topic,
			Data:      payload,
			Timestamp: time.Now(),
		}
		// チャネルが満杯の場合はメッセージをドロップして処理を継続する。
		select {
		case ch <- msg:
		default:
		}
		return nil
	}
	if err := p.client.Subscribe(ctx, topic, handler); err != nil {
		return nil, NewComponentError(p.name, "Subscribe", "failed to subscribe to Redis topic", err)
	}
	return ch, nil
}

// ============================================================
// KafkaPubSub
// ============================================================

// KafkaEventEnvelope は k1s0-messaging の EventEnvelope と互換性を持つ最小限のイベントエンベロープ。
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

// KafkaEventProducer は k1s0-messaging の EventProducer と互換性を持つプロデューサーインターフェース。
// *kafka.Producer を注入することでこのインターフェースを満たせる。
type KafkaEventProducer interface {
	// Publish は指定したイベントエンベロープを Kafka に送信する。
	Publish(ctx context.Context, event KafkaEventEnvelope) error
	// Close はプロデューサーの接続を閉じてリソースを解放する。
	Close() error
}

// KafkaEventHandler は受信した各イベントに対して呼び出されるコールバック関数の型。
type KafkaEventHandler func(ctx context.Context, event KafkaEventEnvelope) error

// KafkaEventConsumer は k1s0-messaging の EventConsumer と互換性を持つコンシューマーインターフェース。
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
	// producer は Kafka へのメッセージ発行を担うプロデューサー実装。
	producer KafkaEventProducer
	// consumer は Kafka からのメッセージ購読を担うコンシューマー実装（nil 許容）。
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
// Message の ID を Kafka メッセージキーとして使用し、Data をペイロードとして送る。
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
		}
		return nil
	}
	if err := p.consumer.Subscribe(ctx, topic, handler); err != nil {
		return nil, NewComponentError(p.name, "Subscribe", "failed to subscribe to Kafka topic", err)
	}
	return ch, nil
}

