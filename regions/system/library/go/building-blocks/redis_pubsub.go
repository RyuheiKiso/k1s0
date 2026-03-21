package buildingblocks

import (
	"context"
	"log/slog"
	"sync"
	"time"
)

// RedisPubSubClient はRedisのPub/Sub操作を担うインターフェース。
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
	// onDropped はメッセージをドロップした際に呼び出すコールバック（G-04 対応）。
	// nil の場合はログのみ出力する。Prometheus カウンターのインクリメント等に使用できる。
	onDropped func(topic string)
}

// RedisPubSubOption は RedisPubSub の生成オプション。
type RedisPubSubOption func(*RedisPubSub)

// WithDroppedMessageCallback はメッセージドロップ時に呼び出すコールバックを設定する（G-04 対応）。
// Prometheus カウンター等のメトリクス収集に使用する。
// 例: WithDroppedMessageCallback(func(topic string) { droppedCounter.WithLabelValues(topic).Inc() })
func WithDroppedMessageCallback(fn func(topic string)) RedisPubSubOption {
	return func(p *RedisPubSub) { p.onDropped = fn }
}

// NewRedisPubSub は新しい RedisPubSub を生成して返す。
// name はコンポーネント識別子、client は Redis Pub/Sub 操作を担うクライアント実装。
// opts により追加設定（ドロップコールバック等）を注入できる。
func NewRedisPubSub(name string, client RedisPubSubClient, opts ...RedisPubSubOption) *RedisPubSub {
	p := &RedisPubSub{
		name:   name,
		client: client,
		status: StatusUninitialized,
	}
	for _, opt := range opts {
		opt(p)
	}
	return p
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
// Redis Pub/Sub はKafkaのようなメッセージヘッダーをサポートしないため、Metadata は送信しない。
func (p *RedisPubSub) Publish(ctx context.Context, msg *Message) error {
	if err := p.client.Publish(ctx, msg.Topic, msg.Data); err != nil {
		return NewComponentError(p.name, "Publish", "failed to publish to Redis", err)
	}
	return nil
}

// Subscribe は指定した Redis トピックのメッセージを受信するチャネルを返す。
// チャネルのバッファサイズは64で、バッファが満杯の場合はメッセージをドロップする。
// backpressure 対策として、コンシューマが追いつかない場合はブロックせずにドロップする。
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
		// バッファ満杯時はメッセージをドロップして配信ループの継続を優先する。
		// コンテキストキャンセル時はエラーを返してシャットダウンを伝播する。
		select {
		case ch <- msg:
		case <-ctx.Done():
			return ctx.Err()
		default:
			// バッファが満杯のためメッセージをドロップする。
			// コンシューマが追いつかない場合の backpressure 対策として、
			// ブロックせずにドロップして配信ループの継続を優先する。
			slog.Warn("PubSub メッセージをドロップしました: バッファ満杯",
				slog.String("topic", topic),
				slog.Int("buffer_cap", cap(ch)),
			)
			// G-04 対応: ドロップイベントをコールバックで通知する（Prometheus カウンター等に使用）。
			if p.onDropped != nil {
				p.onDropped(topic)
			}
		}
		return nil
	}
	if err := p.client.Subscribe(ctx, topic, handler); err != nil {
		return nil, NewComponentError(p.name, "Subscribe", "failed to subscribe to Redis topic", err)
	}
	// コンテキストがキャンセルされたときにチャネルをクローズして購読者にEOFを通知する
	go func() {
		<-ctx.Done()
		close(ch)
	}()
	return ch, nil
}
