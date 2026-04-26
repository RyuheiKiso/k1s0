// 本ファイルは Dapr Pub/Sub building block のアダプタ。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md
//     - PubSub API → Kafka（Dapr Pub/Sub）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/03_PubSub_API.md
//
// リリース時点 placeholder。実 Dapr SDK 接続は plan 04-05 で実装。

package dapr

// 標準 Go ライブラリ。
import (
	// 全 RPC で context を伝搬する。
	"context"
)

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
	// Kafka offset。
	Offset int64
}

// PubSubAdapter は Pub/Sub 操作の interface。
type PubSubAdapter interface {
	// 単発 Publish。
	Publish(ctx context.Context, req PublishRequest) (PublishResponse, error)
}

// daprPubSubAdapter は実装（リリース時点 placeholder）。
type daprPubSubAdapter struct {
	// Dapr Client への参照。
	client *Client
}

// NewPubSubAdapter は PubSubAdapter を生成する。
func NewPubSubAdapter(client *Client) PubSubAdapter {
	// 実装インスタンスを構築する。
	return &daprPubSubAdapter{client: client}
}

// Publish は plan 04-05 で実装。
func (a *daprPubSubAdapter) Publish(_ context.Context, _ PublishRequest) (PublishResponse, error) {
	// placeholder
	return PublishResponse{}, ErrNotWired
}
