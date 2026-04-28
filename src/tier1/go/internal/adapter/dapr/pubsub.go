// 本ファイルは Dapr Pub/Sub building block のアダプタ。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md
//     - PubSub API → Kafka（Dapr Pub/Sub）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/03_PubSub_API.md
//
// 役割（plan 04-05 結線済）:
//   handler.go が呼び出す Publish I/O を封じ込め、Dapr Go SDK の PublishEvent を
//   narrow interface（dapr.go の daprPubSubClient）越しに呼び出す。
//   テナント識別子と冪等性キーは metadata 経由で sidecar に伝搬する。
//
// Kafka offset の扱い:
//   Dapr SDK の PublishEvent は fire-and-forget で Kafka offset を返さないため、
//   PublishResponse.Offset は常に 0 を返す。proto 側の offset フィールドは
//   将来 Dapr が exposing をサポートした際の予約。

package dapr

import (
	// 全 RPC で context を伝搬する。
	"context"

	// Dapr SDK の PublishEvent オプション関数を参照する。
	daprclient "github.com/dapr/go-sdk/client"
)

// metadataKeyIdempotency は Dapr metadata に詰める冪等性キー。
// pubsub component 側で重複検出に使う運用想定（component 設定依存）。
const metadataKeyIdempotency = "idempotencyKey"

// PublishRequest は Publish / BulkPublish 共通の入力。
type PublishRequest struct {
	// Dapr Component 名（pubsub-kafka 等）。
	Component string
	// トピック名（テナント接頭辞付与済）。
	Topic string
	// イベント本文。
	Data []byte
	// Content-Type。
	ContentType string
	// 冪等性キー。
	IdempotencyKey string
	// メタデータ（partition_key 等）。
	Metadata map[string]string
	// テナント識別子。
	TenantID string
}

// PublishResponse は Publish の応答。
type PublishResponse struct {
	// Kafka offset。Dapr SDK は exposing しないため常に 0。
	Offset int64
}

// PubSubAdapter は Pub/Sub 操作の interface。
type PubSubAdapter interface {
	// 単発 Publish。
	Publish(ctx context.Context, req PublishRequest) (PublishResponse, error)
	// Subscribe（server-streaming）: subscription 開始、Receive で逐次イベント取得、Close で終了。
	Subscribe(ctx context.Context, req SubscribeAdapterRequest) (PubSubSubscription, error)
}

// SubscribeAdapterRequest は Subscribe の入力。
type SubscribeAdapterRequest struct {
	Component     string
	Topic         string
	ConsumerGroup string
	TenantID      string
}

// PubSubSubscription は subscription の操作集合。handler は Receive をループし、
// 受信したイベントを gRPC stream 越しにクライアントへ送る。
type PubSubSubscription interface {
	// 次のイベントが届くまで block する。ctx キャンセルで err 返却。
	Receive(ctx context.Context) (*SubscribedEvent, error)
	// subscription を解放する。
	Close() error
}

// SubscribedEvent は 1 件の受信イベント（adapter 中立な中間表現）。
type SubscribedEvent struct {
	// トピック名（テナント prefix 除去済）。
	Topic string
	// 本文。
	Data []byte
	// Content-Type。
	ContentType string
	// メタデータ（ヘッダ）。
	Metadata map[string]string
	// Kafka offset（adapter が分かる場合のみ非 0）。
	Offset int64
	// SDK の ack 関数（成功 ack）。
	Ack func() error
	// SDK の retry 関数（失敗 → DLQ や再配信指示）。
	Retry func() error
}

// daprPubSubAdapter は Client（narrow interface）越しに SDK を呼ぶ実装。
type daprPubSubAdapter struct {
	// Dapr Client への参照。pubsub-用 narrow interface（daprPubSubClient）を持つ。
	client *Client
}

// NewPubSubAdapter は PubSubAdapter を生成する。
func NewPubSubAdapter(client *Client) PubSubAdapter {
	return &daprPubSubAdapter{client: client}
}

// buildPubSubMeta はテナント識別子・冪等性キー・追加 metadata を合成する。
// 呼び出し側で渡された Metadata map を破壊しないよう、新規 map を返す。
func buildPubSubMeta(tenantID, idempotencyKey string, extra map[string]string) map[string]string {
	if tenantID == "" && idempotencyKey == "" && len(extra) == 0 {
		return nil
	}
	// 上書き優先順位: tenantID / idempotencyKey は extra より優先する（adapter 規約）。
	meta := make(map[string]string, len(extra)+2)
	for k, v := range extra {
		meta[k] = v
	}
	if tenantID != "" {
		meta[metadataKeyTenant] = tenantID
	}
	if idempotencyKey != "" {
		meta[metadataKeyIdempotency] = idempotencyKey
	}
	return meta
}

// Subscribe は Dapr SDK Subscribe を呼び、PubSubSubscription を返す。
// ConsumerGroup は Dapr SDK の SubscriptionOptions.Metadata にコンポーネント依存
// キー（kafka なら "consumerGroup"）として詰める運用。
func (a *daprPubSubAdapter) Subscribe(ctx context.Context, req SubscribeAdapterRequest) (PubSubSubscription, error) {
	meta := buildPubSubMeta(req.TenantID, "", nil)
	if req.ConsumerGroup != "" {
		if meta == nil {
			meta = make(map[string]string, 1)
		}
		// kafka backend では "consumerGroup" がコンポーネント既定キー。
		meta["consumerGroup"] = req.ConsumerGroup
	}
	sub, err := a.client.pubsubClient().Subscribe(ctx, daprclient.SubscriptionOptions{
		PubsubName: req.Component,
		Topic:      req.Topic,
		Metadata:   meta,
	})
	if err != nil {
		return nil, err
	}
	return &daprSubscriptionAdapter{sub: sub, topic: req.Topic}, nil
}

// daprSubscriptionAdapter は Dapr SDK Subscription を PubSubSubscription に適合させる。
type daprSubscriptionAdapter struct {
	sub   *daprclient.Subscription
	topic string
}

func (s *daprSubscriptionAdapter) Receive(_ context.Context) (*SubscribedEvent, error) {
	// SDK の Receive() は ctx を取らない。stream 終了 (Close) まで block する。
	msg, err := s.sub.Receive()
	if err != nil {
		return nil, err
	}
	if msg == nil || msg.TopicEvent == nil {
		return nil, nil
	}
	// TopicEvent.Data は interface{}（JSON decoded など）。raw bytes を取る場合は
	// RawData を優先する（CloudEvents の binary 解放経路）。
	var data []byte
	if msg.RawData != nil {
		data = msg.RawData
	} else if b, ok := msg.Data.([]byte); ok {
		data = b
	} else if s, ok := msg.Data.(string); ok {
		data = []byte(s)
	}
	return &SubscribedEvent{
		Topic:       s.topic,
		Data:        data,
		ContentType: msg.DataContentType,
		Metadata:    nil,    // SDK の TopicEvent には header メタデータ未露出
		Offset:      0,      // SDK は exposing しない
		Ack:         msg.Success,
		Retry:       msg.Retry,
	}, nil
}

func (s *daprSubscriptionAdapter) Close() error {
	return s.sub.Close()
}

// Publish はトピックへ event を発行する。
func (a *daprPubSubAdapter) Publish(ctx context.Context, req PublishRequest) (PublishResponse, error) {
	// metadata 構築（テナント + 冪等性 + 利用側追加）。
	meta := buildPubSubMeta(req.TenantID, req.IdempotencyKey, req.Metadata)

	// SDK の PublishEvent オプションを組み立てる。
	// content-type が空でも SDK は default を使うので無条件指定はしない。
	opts := make([]daprclient.PublishEventOption, 0, 2)
	if req.ContentType != "" {
		opts = append(opts, daprclient.PublishEventWithContentType(req.ContentType))
	}
	if len(meta) > 0 {
		opts = append(opts, daprclient.PublishEventWithMetadata(meta))
	}

	// Dapr SDK 呼び出し。data は []byte で渡し、SDK 側で適切に serialize される。
	if err := a.client.pubsubClient().PublishEvent(ctx, req.Component, req.Topic, req.Data, opts...); err != nil {
		return PublishResponse{}, err
	}
	// SDK は Kafka offset を返さないため 0 を返却する（proto field は予約）。
	return PublishResponse{Offset: 0}, nil
}
