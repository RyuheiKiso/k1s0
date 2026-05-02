// k1s0 Audit ラッパー。
//
// SDK の AuditClient.Record / Query を per-request tenant 伝搬付きで露出する。
// VerifyChain は管理者ツールの責務であり BFF からは公開しない。
//
// Query 結果は SDK が返す proto 型（auditv1.AuditEvent）を BFF JSON 応答用の
// 軽量構造体に詰め替える。これは「SDK 型を上位層に漏らさない」境界保護のため。

package k1s0client

// 標準 / 内部 import。
import (
	// context 伝搬。
	"context"
	// 時刻。
	"time"
)

// AuditEventSummary は BFF JSON 応答用の監査イベント要約（proto 型を露出させない）。
type AuditEventSummary struct {
	// 発生時刻（UnixMilli）。
	OccurredAtMillis int64
	// 行為主体（subject）。
	Actor string
	// 動作（CREATE / READ / UPDATE / DELETE 等）。
	Action string
	// 対象リソース（URI 形式）。
	Resource string
	// 結果（SUCCESS / DENIED / ERROR）。
	Outcome string
	// 任意メタデータ（PII Mask 適用後）。
	Attributes map[string]string
}

// AuditRecord は監査イベントを記録する。idempotencyKey が空なら毎回新エントリを作る。
func (c *Client) AuditRecord(ctx context.Context, actor, action, resource, outcome string, attributes map[string]string, idempotencyKey string) (auditID string, err error) {
	// SDK facade を呼ぶ。
	return c.client.Audit().Record(withTenantFromRequest(ctx), actor, action, resource, outcome, attributes, idempotencyKey)
}

// AuditQuery は監査イベントを範囲検索する。出力は PII Mask 自動適用済。
func (c *Client) AuditQuery(ctx context.Context, from, to time.Time, filters map[string]string, limit int32) ([]AuditEventSummary, error) {
	// SDK facade を呼ぶ。
	events, err := c.client.Audit().Query(withTenantFromRequest(ctx), from, to, filters, limit)
	if err != nil {
		return nil, err
	}
	// proto 型を BFF 用の軽量構造体に詰め替える。
	out := make([]AuditEventSummary, 0, len(events))
	for _, e := range events {
		// timestamp は nil ガード後に UnixMilli へ変換する。
		var occurredAt int64
		if ts := e.GetTimestamp(); ts != nil {
			occurredAt = ts.AsTime().UnixMilli()
		}
		out = append(out, AuditEventSummary{
			OccurredAtMillis: occurredAt,
			Actor:            e.GetActor(),
			Action:           e.GetAction(),
			Resource:         e.GetResource(),
			Outcome:          e.GetOutcome(),
			Attributes:       e.GetAttributes(),
		})
	}
	return out, nil
}
