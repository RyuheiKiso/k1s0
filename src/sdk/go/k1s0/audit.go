// 本ファイルは k1s0 Go SDK の Audit 動詞統一 facade。
package k1s0

import (
	"context"
	"time"

	auditv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/audit/v1"
	"google.golang.org/protobuf/types/known/timestamppb"
)

// AuditClient は AuditService の動詞統一 facade。
type AuditClient struct{ client *Client }

// Audit は親 Client から AuditClient を返す。
func (c *Client) Audit() *AuditClient { return c.audit }

// Record は監査イベント記録。WORM ストアに append-only、audit_id を返す。
func (a *AuditClient) Record(ctx context.Context, actor, action, resource, outcome string, attributes map[string]string) (string, error) {
	resp, e := a.client.raw.Audit.Record(ctx, &auditv1.RecordAuditRequest{
		Event: &auditv1.AuditEvent{
			Timestamp:  timestamppb.New(time.Now().UTC()),
			Actor:      actor,
			Action:     action,
			Resource:   resource,
			Outcome:    outcome,
			Attributes: attributes,
		},
		Context: a.client.tenantContext(),
	})
	if e != nil {
		return "", e
	}
	return resp.GetAuditId(), nil
}

// Query は監査イベント検索。範囲 + filter で取得、出力には PII Mask が自動適用される。
func (a *AuditClient) Query(ctx context.Context, from, to time.Time, filters map[string]string, limit int32) ([]*auditv1.AuditEvent, error) {
	resp, e := a.client.raw.Audit.Query(ctx, &auditv1.QueryAuditRequest{
		From:    timestamppb.New(from),
		To:      timestamppb.New(to),
		Filters: filters,
		Limit:   limit,
		Context: a.client.tenantContext(),
	})
	if e != nil {
		return nil, e
	}
	return resp.GetEvents(), nil
}
