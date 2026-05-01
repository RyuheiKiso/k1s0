// k1s0 Secrets ラッパー。
//
// SDK の SecretsClient.Get / Rotate を per-request tenant 伝搬付きで露出する。
// GetDynamic は BFF から使わない想定（短命クレデンシャルは tier2 で取得）。

package k1s0client

// 標準 / 内部 import。
import (
	// context 伝搬。
	"context"

	// SDK 高水準 facade（オプション関数を参照するため）。
	"github.com/k1s0/sdk-go/k1s0"
)

// SecretsGet は k1s0 Secrets から指定 secret を取得する。
// 戻り値の values は key/value マップ（VAULT 互換）、version は最新版番号。
func (c *Client) SecretsGet(ctx context.Context, name string) (values map[string]string, version int32, err error) {
	// SDK facade を呼ぶ。
	return c.client.Secrets().Get(withTenantFromRequest(ctx), name)
}

// SecretsRotate は指定 secret をローテートする。
// gracePeriodSec / idempotencyKey は省略可能（0 / "" でオプションをスキップ）。
func (c *Client) SecretsRotate(ctx context.Context, name string, gracePeriodSec int32, idempotencyKey string) (newVersion, previousVersion int32, err error) {
	// SDK のオプション列を組み立てる。
	var opts []k1s0.RotateOption
	// gracePeriod が正の場合のみ option を追加する。
	if gracePeriodSec > 0 {
		opts = append(opts, k1s0.WithGracePeriod(gracePeriodSec))
	}
	// idempotencyKey が指定された場合のみ option を追加する。
	if idempotencyKey != "" {
		opts = append(opts, k1s0.WithIdempotencyKeyRotate(idempotencyKey))
	}
	// SDK facade を呼ぶ。
	return c.client.Secrets().Rotate(withTenantFromRequest(ctx), name, opts...)
}
