// k1s0 PubSub ラッパー。
//
// SDK の PubSubClient.Publish を per-request tenant 伝搬付きで露出する。
// idempotencyKey / metadata は省略可能。Subscribe は BFF からは使わない（コンシューマは tier2）。

package k1s0client

// 標準 / 内部 import。
import (
	// context 伝搬。
	"context"

	// SDK 高水準 facade（オプション関数を参照するため）。
	"github.com/k1s0/sdk-go/k1s0"
)

// PubSubPublish は k1s0 PubSub にメッセージを発行する。
// idempotencyKey / metadata が空ならオプション付与をスキップする。
func (c *Client) PubSubPublish(ctx context.Context, topic string, data []byte, contentType, idempotencyKey string, metadata map[string]string) (offset int64, err error) {
	// SDK のオプション列を組み立てる。
	var opts []k1s0.PublishOption
	// idempotencyKey が指定された場合のみ option を追加する。
	if idempotencyKey != "" {
		opts = append(opts, k1s0.WithIdempotencyKey(idempotencyKey))
	}
	// metadata が指定された場合のみ option を追加する。
	if len(metadata) > 0 {
		opts = append(opts, k1s0.WithMetadata(metadata))
	}
	// SDK facade を呼ぶ。
	return c.client.PubSub().Publish(withTenantFromRequest(ctx), topic, data, contentType, opts...)
}
