// k1s0 Feature ラッパー。
//
// SDK の FeatureClient.EvaluateBoolean を per-request tenant 伝搬付きで露出する。
// String / Number / Object / Admin (RegisterFlag 等) は BFF 公開対象外（必要時に拡張）。

package k1s0client

// 標準 import。
import (
	// context 伝搬。
	"context"
)

// FeatureEvaluateBoolean は Boolean 型 Feature Flag を評価する。
// evalCtx は OpenFeature の Evaluation Context に相当（テナント / ユーザ属性等）。
// 戻り値の variant は flag バリアント名、reason は flagd の評価理由（TARGETING_MATCH 等）。
func (c *Client) FeatureEvaluateBoolean(ctx context.Context, flagKey string, evalCtx map[string]string) (value bool, variant, reason string, err error) {
	// SDK facade を呼ぶ。
	return c.client.Feature().EvaluateBoolean(withTenantFromRequest(ctx), flagKey, evalCtx)
}
