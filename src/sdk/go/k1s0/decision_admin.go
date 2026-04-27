// 本ファイルは k1s0 Go SDK の DecisionAdmin 動詞統一 facade。
// JDM ルール文書の登録 / バージョン一覧 / 取得を提供する。
package k1s0

import (
	"context"

	decisionv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/decision/v1"
)

// DecisionAdminClient は DecisionAdminService の動詞統一 facade。
type DecisionAdminClient struct{ client *Client }

// DecisionAdmin は親 Client から DecisionAdminClient を返す。
func (c *Client) DecisionAdmin() *DecisionAdminClient { return c.decisionAdmin }

// RegisterRule は JDM 文書の登録。返り値は (rule_version, effective_at_ms)。
func (d *DecisionAdminClient) RegisterRule(ctx context.Context, ruleID string, jdmDocument, sigstoreSignature []byte, commitHash string) (ruleVersion string, effectiveAtMs int64, err error) {
	resp, e := d.client.raw.DecisionAdmin.RegisterRule(ctx, &decisionv1.RegisterRuleRequest{
		RuleId:            ruleID,
		JdmDocument:       jdmDocument,
		SigstoreSignature: sigstoreSignature,
		CommitHash:        commitHash,
		Context:           d.client.tenantContext(),
	})
	if e != nil {
		return "", 0, e
	}
	return resp.GetRuleVersion(), resp.GetEffectiveAtMs(), nil
}

// ListVersions はルールバージョン一覧を返す（登録時刻昇順）。
func (d *DecisionAdminClient) ListVersions(ctx context.Context, ruleID string) ([]*decisionv1.RuleVersionMeta, error) {
	resp, e := d.client.raw.DecisionAdmin.ListVersions(ctx, &decisionv1.ListVersionsRequest{
		RuleId:  ruleID,
		Context: d.client.tenantContext(),
	})
	if e != nil {
		return nil, e
	}
	return resp.GetVersions(), nil
}

// GetRule は特定バージョンの JDM 文書とメタ情報を返す。
func (d *DecisionAdminClient) GetRule(ctx context.Context, ruleID, ruleVersion string) (jdmDocument []byte, meta *decisionv1.RuleVersionMeta, err error) {
	resp, e := d.client.raw.DecisionAdmin.GetRule(ctx, &decisionv1.GetRuleRequest{
		RuleId:      ruleID,
		RuleVersion: ruleVersion,
		Context:     d.client.tenantContext(),
	})
	if e != nil {
		return nil, nil, e
	}
	return resp.GetJdmDocument(), resp.GetMeta(), nil
}
