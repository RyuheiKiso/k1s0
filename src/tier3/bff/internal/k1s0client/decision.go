// k1s0 Decision ラッパー。
//
// SDK の DecisionClient.Evaluate を per-request tenant 伝搬付きで露出する。
// BatchEvaluate / Admin (RegisterRule 等) は BFF からは使わない想定。

package k1s0client

// 標準 import。
import (
	// context 伝搬。
	"context"
)

// DecisionEvaluate は JDM ルール評価を行う。
// inputJSON は評価コンテキスト、includeTrace=true なら traceJSON にトレース情報が入る。
func (c *Client) DecisionEvaluate(ctx context.Context, ruleID, ruleVersion string, inputJSON []byte, includeTrace bool) (outputJSON, traceJSON []byte, elapsedUs int64, err error) {
	// SDK facade を呼ぶ。
	return c.client.Decision().Evaluate(withTenantFromRequest(ctx), ruleID, ruleVersion, inputJSON, includeTrace)
}
