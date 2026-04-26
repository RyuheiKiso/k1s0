// 本ファイルは t1-state Pod の FeatureService 4 RPC ハンドラ実装。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/11_Feature_API.md
//
// scope（リリース時点 placeholder）: 実 flagd 結線は plan 04-13。
// FeatureAdminService（RegisterFlag / GetFlag / ListFlags）は本リリース時点 未登録（採用初期で追加）。

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
func (h *featureHandler) EvaluateBoolean(ctx context.Context, req *featurev1.EvaluateRequest) (*featurev1.BooleanResponse, error) {
	// 入力 nil 防御。
	if req == nil {
		// 不正引数返却。
		return nil, status.Error(codes.InvalidArgument, "tier1/feature: nil request")
	}
	// adapter 入力。
	areq := makeFlagReq(req)
	// adapter 呼出。
	aresp, err := h.deps.FeatureAdapter.EvaluateBoolean(ctx, areq)
	// エラー翻訳。
	if err != nil {
		// 翻訳して返却。
		return nil, translateFeatureErr(err, "EvaluateBoolean")
	}
	// 応答変換。
	return &featurev1.BooleanResponse{
		// 評価値。
		Value: aresp.Value,
		// メタ情報。
		Metadata: makeFlagMeta(aresp.Variant, aresp.Reason),
	}, nil
}

// EvaluateString は string Flag 評価。
func (h *featureHandler) EvaluateString(ctx context.Context, req *featurev1.EvaluateRequest) (*featurev1.StringResponse, error) {
	// 入力 nil 防御。
	if req == nil {
		// 不正引数返却。
		return nil, status.Error(codes.InvalidArgument, "tier1/feature: nil request")
	}
	// adapter 入力構築 + 呼出。
	aresp, err := h.deps.FeatureAdapter.EvaluateString(ctx, makeFlagReq(req))
	// エラー翻訳。
	if err != nil {
		// 翻訳して返却。
		return nil, translateFeatureErr(err, "EvaluateString")
	}
	// 応答変換。
	return &featurev1.StringResponse{
		// 評価値。
		Value: aresp.Value,
		// メタ情報。
		Metadata: makeFlagMeta(aresp.Variant, aresp.Reason),
	}, nil
}

// EvaluateNumber は number Flag 評価。
func (h *featureHandler) EvaluateNumber(ctx context.Context, req *featurev1.EvaluateRequest) (*featurev1.NumberResponse, error) {
	// 入力 nil 防御。
	if req == nil {
		// 不正引数返却。
		return nil, status.Error(codes.InvalidArgument, "tier1/feature: nil request")
	}
	// adapter 呼出。
	aresp, err := h.deps.FeatureAdapter.EvaluateNumber(ctx, makeFlagReq(req))
	// エラー翻訳。
	if err != nil {
		// 翻訳して返却。
		return nil, translateFeatureErr(err, "EvaluateNumber")
	}
	// 応答変換。
	return &featurev1.NumberResponse{
		// 評価値。
		Value: aresp.Value,
		// メタ情報。
		Metadata: makeFlagMeta(aresp.Variant, aresp.Reason),
	}, nil
}

// EvaluateObject は object Flag 評価。
func (h *featureHandler) EvaluateObject(ctx context.Context, req *featurev1.EvaluateRequest) (*featurev1.ObjectResponse, error) {
	// 入力 nil 防御。
	if req == nil {
		// 不正引数返却。
		return nil, status.Error(codes.InvalidArgument, "tier1/feature: nil request")
	}
	// adapter 呼出。
	aresp, err := h.deps.FeatureAdapter.EvaluateObject(ctx, makeFlagReq(req))
	// エラー翻訳。
	if err != nil {
		// 翻訳して返却。
		return nil, translateFeatureErr(err, "EvaluateObject")
	}
	// 応答変換。
	return &featurev1.ObjectResponse{
		// 評価値（JSON シリアライズ済 bytes）。
		ValueJson: aresp.ValueJSON,
		// メタ情報。
		Metadata: makeFlagMeta(aresp.Variant, aresp.Reason),
	}, nil
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
	// ErrNotWired → Unimplemented。
	if isNotWired(err) {
		// 翻訳メッセージ。
		return status.Errorf(codes.Unimplemented, "tier1/feature: %s not yet wired to flagd (plan 04-13)", rpc)
	}
	// その他 → Internal。
	return status.Errorf(codes.Internal, "tier1/feature: %s adapter error: %v", rpc, err)
}
