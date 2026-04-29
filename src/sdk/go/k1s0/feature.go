// 本ファイルは k1s0 Go SDK の Feature 動詞統一 facade（評価部のみ）。
// FeatureAdminService は raw 経由でアクセス。
package k1s0

import (
	"context"

	featurev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/feature/v1"
)

// FeatureClient は FeatureService の動詞統一 facade。
type FeatureClient struct{ client *Client }

// Feature は親 Client から FeatureClient を返す。
func (c *Client) Feature() *FeatureClient { return c.feature }

// makeReq は flag 評価リクエストを構築する共通 helper。
func (f *FeatureClient) makeReq(ctx context.Context, flagKey string, evalCtx map[string]string) *featurev1.EvaluateRequest {
	return &featurev1.EvaluateRequest{
		FlagKey:           flagKey,
		EvaluationContext: evalCtx,
		Context:           f.client.tenantContext(ctx),
	}
}

// EvaluateBoolean は boolean Flag 評価。値 + variant + reason を返す。
func (f *FeatureClient) EvaluateBoolean(ctx context.Context, flagKey string, evalCtx map[string]string) (value bool, variant, reason string, err error) {
	resp, e := f.client.raw.Feature.EvaluateBoolean(ctx, f.makeReq(ctx, flagKey, evalCtx))
	if e != nil {
		return false, "", "", e
	}
	return resp.GetValue(), resp.GetMetadata().GetVariant(), resp.GetMetadata().GetReason(), nil
}

// EvaluateString は string Flag 評価。
func (f *FeatureClient) EvaluateString(ctx context.Context, flagKey string, evalCtx map[string]string) (value, variant, reason string, err error) {
	resp, e := f.client.raw.Feature.EvaluateString(ctx, f.makeReq(ctx, flagKey, evalCtx))
	if e != nil {
		return "", "", "", e
	}
	return resp.GetValue(), resp.GetMetadata().GetVariant(), resp.GetMetadata().GetReason(), nil
}

// EvaluateNumber は number Flag 評価。
func (f *FeatureClient) EvaluateNumber(ctx context.Context, flagKey string, evalCtx map[string]string) (value float64, variant, reason string, err error) {
	resp, e := f.client.raw.Feature.EvaluateNumber(ctx, f.makeReq(ctx, flagKey, evalCtx))
	if e != nil {
		return 0, "", "", e
	}
	return resp.GetValue(), resp.GetMetadata().GetVariant(), resp.GetMetadata().GetReason(), nil
}

// EvaluateObject は object Flag 評価（JSON シリアライズ済 bytes）。
func (f *FeatureClient) EvaluateObject(ctx context.Context, flagKey string, evalCtx map[string]string) (valueJSON []byte, variant, reason string, err error) {
	resp, e := f.client.raw.Feature.EvaluateObject(ctx, f.makeReq(ctx, flagKey, evalCtx))
	if e != nil {
		return nil, "", "", e
	}
	return resp.GetValueJson(), resp.GetMetadata().GetVariant(), resp.GetMetadata().GetReason(), nil
}
