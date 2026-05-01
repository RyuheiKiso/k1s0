// 本ファイルは t1-state Pod の FeatureService 4 RPC ハンドラ実装。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/11_Feature_API.md
//
// scope: 実 flagd 結線は OpenFeature SDK 経由で確立可能。dev / CI では in-memory
// Configuration backend 経由で評価される。FeatureAdminService（RegisterFlag /
// GetFlag / ListFlags）は同 Pod 内に併設実装（feature_admin.go）。

package state

// 標準 / 内部パッケージ。
import (
	// context 伝搬。
	"context"
	// Dapr / flagd adapter。
	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
	// SDK 生成 stub の FeatureService 型。
	featurev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/feature/v1"
	// gRPC エラーコード。
	"google.golang.org/grpc/codes"
	// gRPC ステータスエラー。
	"google.golang.org/grpc/status"
)

// featureHandler は FeatureService の handler 実装。
type featureHandler struct {
	// 将来 RPC 用埋め込み。
	featurev1.UnimplementedFeatureServiceServer
	// adapter 集合。
	deps Deps
}

// EvaluateBoolean は boolean Flag 評価。
//
// FR-T1-FEATURE-001 受け入れ基準: 「評価失敗時はデフォルト値を返す（K1s0Error にしない、
// 業務継続優先）」を満たすため、adapter エラーは zero-value + Reason="ERROR" でフォールバック。
// PermissionDenied / InvalidArgument / Unauthenticated 系は業務ロジック誤りなので gRPC error 維持。
func (h *featureHandler) EvaluateBoolean(ctx context.Context, req *featurev1.EvaluateRequest) (*featurev1.BooleanResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "tier1/feature: nil request")
	}
	if _, err := requireTenantIDFromCtx(ctx, req.GetContext(), "Feature.EvaluateBoolean"); err != nil {
		return nil, err
	}
	aresp, err := h.deps.FeatureAdapter.EvaluateBoolean(ctx, makeFlagReq(req))
	if err != nil {
		if isClientFault(err) {
			return nil, translateFeatureErr(err, "EvaluateBoolean")
		}
		return &featurev1.BooleanResponse{Value: false, Metadata: makeFlagMeta("default", "ERROR")}, nil
	}
	return &featurev1.BooleanResponse{
		Value:    aresp.Value,
		Metadata: makeFlagMeta(aresp.Variant, aresp.Reason),
	}, nil
}

// EvaluateString は string Flag 評価。FR-T1-FEATURE-001 業務継続優先で fail-soft。
func (h *featureHandler) EvaluateString(ctx context.Context, req *featurev1.EvaluateRequest) (*featurev1.StringResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "tier1/feature: nil request")
	}
	if _, err := requireTenantIDFromCtx(ctx, req.GetContext(), "Feature.EvaluateString"); err != nil {
		return nil, err
	}
	aresp, err := h.deps.FeatureAdapter.EvaluateString(ctx, makeFlagReq(req))
	if err != nil {
		if isClientFault(err) {
			return nil, translateFeatureErr(err, "EvaluateString")
		}
		return &featurev1.StringResponse{Value: "", Metadata: makeFlagMeta("default", "ERROR")}, nil
	}
	return &featurev1.StringResponse{
		Value:    aresp.Value,
		Metadata: makeFlagMeta(aresp.Variant, aresp.Reason),
	}, nil
}

// EvaluateNumber は number Flag 評価。FR-T1-FEATURE-001 業務継続優先で fail-soft。
func (h *featureHandler) EvaluateNumber(ctx context.Context, req *featurev1.EvaluateRequest) (*featurev1.NumberResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "tier1/feature: nil request")
	}
	if _, err := requireTenantIDFromCtx(ctx, req.GetContext(), "Feature.EvaluateNumber"); err != nil {
		return nil, err
	}
	aresp, err := h.deps.FeatureAdapter.EvaluateNumber(ctx, makeFlagReq(req))
	if err != nil {
		if isClientFault(err) {
			return nil, translateFeatureErr(err, "EvaluateNumber")
		}
		return &featurev1.NumberResponse{Value: 0, Metadata: makeFlagMeta("default", "ERROR")}, nil
	}
	return &featurev1.NumberResponse{
		Value:    aresp.Value,
		Metadata: makeFlagMeta(aresp.Variant, aresp.Reason),
	}, nil
}

// EvaluateObject は object Flag 評価。FR-T1-FEATURE-001 業務継続優先で fail-soft。
func (h *featureHandler) EvaluateObject(ctx context.Context, req *featurev1.EvaluateRequest) (*featurev1.ObjectResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "tier1/feature: nil request")
	}
	if _, err := requireTenantIDFromCtx(ctx, req.GetContext(), "Feature.EvaluateObject"); err != nil {
		return nil, err
	}
	aresp, err := h.deps.FeatureAdapter.EvaluateObject(ctx, makeFlagReq(req))
	if err != nil {
		if isClientFault(err) {
			return nil, translateFeatureErr(err, "EvaluateObject")
		}
		return &featurev1.ObjectResponse{ValueJson: nil, Metadata: makeFlagMeta("default", "ERROR")}, nil
	}
	return &featurev1.ObjectResponse{
		ValueJson: aresp.ValueJSON,
		Metadata:  makeFlagMeta(aresp.Variant, aresp.Reason),
	}, nil
}

// isClientFault は呼出側起因のエラー（InvalidArgument / Unauthenticated / PermissionDenied）
// を判定する。これらは fail-soft しない（業務継続優先の対象外）。
func isClientFault(err error) bool {
	st, ok := status.FromError(err)
	if !ok {
		return false
	}
	switch st.Code() {
	case codes.InvalidArgument, codes.Unauthenticated, codes.PermissionDenied:
		return true
	default:
		return false
	}
}

// makeFlagReq は proto Request → adapter Request 変換ヘルパ。
func makeFlagReq(req *featurev1.EvaluateRequest) dapr.FlagEvaluateRequest {
	// adapter 入力を構築する。
	return dapr.FlagEvaluateRequest{
		// Flag キー。
		FlagKey: req.GetFlagKey(),
		// 評価コンテキスト。
		EvaluationContext: req.GetEvaluationContext(),
		// テナント。
		TenantID: tenantIDOf(req.GetContext()),
	}
}

// makeFlagMeta は variant / reason から FlagMetadata proto を組み立てるヘルパ。
func makeFlagMeta(variant, reason string) *featurev1.FlagMetadata {
	// FlagMetadata を返却する（kind は本リリース時点 では未設定、plan 04-13 で確定）。
	return &featurev1.FlagMetadata{
		// バリアント名。
		Variant: variant,
		// 評価理由。
		Reason: reason,
	}
}

// translateFeatureErr は Feature 用エラー翻訳。
func translateFeatureErr(err error, rpc string) error {
	// dapr が返す gRPC status を尊重する（例: configuration store 未設定 →
	// FailedPrecondition、permission 系 → PermissionDenied）。`status.FromError` は
	// gRPC status を保持していれば true を返し、保持していなければ Unknown 扱い。
	// ここでは Unknown 以外（つまり真正な gRPC status）はそのまま再 emit して
	// HTTP layer で適切な status code（FailedPrecondition→409 / Unavailable→503 等）に
	// マップさせる。これにより adapter が透過的に上流のエラーを伝搬できる。
	if st, ok := status.FromError(err); ok && st.Code() != codes.Unknown && st.Code() != codes.OK {
		return status.Errorf(st.Code(), "tier1/feature: %s adapter error: %s", rpc, st.Message())
	}
	// その他 → Internal。
	return status.Errorf(codes.Internal, "tier1/feature: %s adapter error: %v", rpc, err)
}
