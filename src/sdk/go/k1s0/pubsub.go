// 本ファイルは k1s0 Go SDK の PubSub 動詞統一 facade。
// `k1s0.PubSub().Publish(...)` 形式で PubSubService への呼出を提供する。

package k1s0

// 標準 / 内部 import。
import (
	// context 伝搬。
	"context"
	// SDK 生成 stub の PubSubService 型。
	commonv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/common/v1"
	pubsubv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/pubsub/v1"
)

// PubSubClient は PubSubService の動詞統一 facade。
type PubSubClient struct {
	// 親 Client への参照。
	client *Client
}

// PublishOption は Publish の任意パラメータを設定する。
type PublishOption func(*pubsubv1.PublishRequest)

// WithIdempotencyKey は冪等性キーを Publish に渡す（24h 重複抑止）。
func WithIdempotencyKey(key string) PublishOption {
	// クロージャで PublishRequest を変更する。
	return func(req *pubsubv1.PublishRequest) {
		// 冪等性キー設定。
		req.IdempotencyKey = key
	}
}

// WithMetadata は Publish にメタデータを追加する（partition_key 等）。
func WithMetadata(metadata map[string]string) PublishOption {
	// クロージャで metadata を上書きする。
	return func(req *pubsubv1.PublishRequest) {
		// メタデータマップを設定する。
		req.Metadata = metadata
	}
}

// Publish は単発 Publish。Kafka offset を返す。
func (p *PubSubClient) Publish(ctx context.Context, topic string, data []byte, contentType string, opts ...PublishOption) (offset int64, err error) {
	// proto Request を構築する。
	req := &pubsubv1.PublishRequest{
		// トピック名（テナント prefix は tier1 が自動付与）。
		Topic: topic,
		// データ本文。
		Data: data,
		// Content-Type。
		ContentType: contentType,
		// TenantContext を継承する。
		Context: &commonv1.TenantContext{
			// テナント ID。
			TenantId: p.client.cfg.TenantID,
			// subject。
			Subject: p.client.cfg.Subject,
		},
	}
	// 各 PublishOption を req に適用する。
	for _, opt := range opts {
		// クロージャを呼び出して req を変更する。
		opt(req)
	}
	// 生成 stub 経由で RPC 呼び出し。
	resp, e := p.client.raw.PubSub.Publish(ctx, req)
	// gRPC エラー時はそのまま伝搬する。
	if e != nil {
		// caller に error を返却する。
		return 0, e
	}
	// offset を返却する。
	return resp.GetOffset(), nil
}
