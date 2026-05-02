// k1s0 Workflow ラッパー。
//
// SDK の WorkflowClient.Start を per-request tenant 伝搬付きで露出する。
// Signal / Query / Cancel / Terminate / GetStatus は BFF からの利用頻度が低いため
// リリース時点では Start のみを公開する（必要時に拡張）。

package k1s0client

// 標準 import。
import (
	// context 伝搬。
	"context"
)

// WorkflowStart はワークフロー開始。
// idempotent=true で同 workflowID 再投入を冪等化する（Dapr / Temporal バックエンド両対応）。
func (c *Client) WorkflowStart(ctx context.Context, workflowType, workflowID string, input []byte, idempotent bool) (returnedID, runID string, err error) {
	// SDK facade を呼ぶ。
	return c.client.Workflow().Start(withTenantFromRequest(ctx), workflowType, workflowID, input, idempotent)
}
