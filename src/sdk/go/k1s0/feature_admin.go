// 本ファイルは k1s0 Go SDK の FeatureAdmin 動詞統一 facade。
// Flag 定義の登録 / 取得 / 一覧を提供する。
package k1s0

import (
	"context"

	featurev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/feature/v1"
)

// FeatureAdminClient は FeatureAdminService の動詞統一 facade。
type FeatureAdminClient struct{ client *Client }

// FeatureAdmin は親 Client から FeatureAdminClient を返す。
func (c *Client) FeatureAdmin() *FeatureAdminClient { return c.featureAdmin }

// RegisterFlag は Flag 定義の登録。permission 種別は approval_id 必須。
// 返り値は flag_key 内で単調増加するバージョン。
func (f *FeatureAdminClient) RegisterFlag(ctx context.Context, flag *featurev1.FlagDefinition, changeReason, approvalID string) (int64, error) {
	resp, e := f.client.raw.FeatureAdmin.RegisterFlag(ctx, &featurev1.RegisterFlagRequest{
		Flag:         flag,
		ChangeReason: changeReason,
		ApprovalId:   approvalID,
		Context:      f.client.tenantContext(),
	})
	if e != nil {
		return 0, e
	}
	return resp.GetVersion(), nil
}

// GetFlag は Flag 定義の取得。version=0 は最新を意味する（proto3 optional）。
func (f *FeatureAdminClient) GetFlag(ctx context.Context, flagKey string, version int64) (*featurev1.FlagDefinition, int64, error) {
	req := &featurev1.GetFlagRequest{
		FlagKey: flagKey,
		Context: f.client.tenantContext(),
	}
	// version > 0 のときだけ optional を設定する。
	if version > 0 {
		req.Version = &version
	}
	resp, e := f.client.raw.FeatureAdmin.GetFlag(ctx, req)
	if e != nil {
		return nil, 0, e
	}
	return resp.GetFlag(), resp.GetVersion(), nil
}

// ListFlags は Flag 定義一覧。kind / state は nil で全件、指定すれば絞込。
func (f *FeatureAdminClient) ListFlags(ctx context.Context, kind *featurev1.FlagKind, state *featurev1.FlagState) ([]*featurev1.FlagDefinition, error) {
	req := &featurev1.ListFlagsRequest{Context: f.client.tenantContext()}
	if kind != nil {
		req.Kind = kind
	}
	if state != nil {
		req.State = state
	}
	resp, e := f.client.raw.FeatureAdmin.ListFlags(ctx, req)
	if e != nil {
		return nil, e
	}
	return resp.GetFlags(), nil
}
