// 本ファイルは k1s0 Go SDK の Decision 動詞統一 facade（評価部のみ）。
// DecisionAdminService（RegisterRule / ListVersions / GetRule）は raw 経由。
package k1s0

import (
	"context"

	decisionv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/decision/v1"
)

// DecisionClient は DecisionService（評価）の動詞統一 facade。
type DecisionClient struct{ client *Client }

// Decision は親 Client から DecisionClient を返す。
func (c *Client) Decision() *DecisionClient { return c.decision }

// Evaluate はルール評価（同期）。output_json と elapsed_us を返す。
func (d *DecisionClient) Evaluate(ctx context.Context, ruleID, ruleVersion string, inputJSON []byte, includeTrace bool) (outputJSON, traceJSON []byte, elapsedUs int64, err error) {
	resp, e := d.client.raw.Decision.Evaluate(ctx, &decisionv1.EvaluateRequest{
		RuleId:       ruleID,
		RuleVersion:  ruleVersion,
		InputJson:    inputJSON,
		IncludeTrace: includeTrace,
		Context:      d.client.tenantContext(ctx),
	})
	if e != nil {
		return nil, nil, 0, e
	}
	return resp.GetOutputJson(), resp.GetTraceJson(), resp.GetElapsedUs(), nil
}

// BatchEvaluate はバッチ評価（複数 input を一括）。
func (d *DecisionClient) BatchEvaluate(ctx context.Context, ruleID, ruleVersion string, inputs [][]byte) ([][]byte, error) {
	resp, e := d.client.raw.Decision.BatchEvaluate(ctx, &decisionv1.BatchEvaluateRequest{
		RuleId:      ruleID,
		RuleVersion: ruleVersion,
		InputsJson:  inputs,
		Context:     d.client.tenantContext(ctx),
	})
	if e != nil {
		return nil, e
	}
	return resp.GetOutputsJson(), nil
}
