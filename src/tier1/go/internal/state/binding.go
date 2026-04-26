// 本ファイルは t1-state Pod の BindingService 1 RPC ハンドラ実装。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/05_Binding_API.md
//
// scope（リリース時点 placeholder）: 実 Dapr Output Binding 結線は plan 04-12。

package state

// 標準 / 内部パッケージ。
import (
	// context 伝搬。
	"context"
	// Dapr adapter。
	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
	// SDK 生成 stub の BindingService 型。
	bindingv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/binding/v1"
	// gRPC エラーコード。
	"google.golang.org/grpc/codes"
	// gRPC ステータスエラー。
	"google.golang.org/grpc/status"
)

// bindingHandler は BindingService の handler 実装。
type bindingHandler struct {
	// 将来 RPC 用埋め込み。
	bindingv1.UnimplementedBindingServiceServer
	// adapter 集合。
	deps Deps
}

// Invoke は出力バインディング呼出。
func (h *bindingHandler) Invoke(ctx context.Context, req *bindingv1.InvokeBindingRequest) (*bindingv1.InvokeBindingResponse, error) {
	// 入力 nil 防御。
	if req == nil {
		// 不正引数返却。
		return nil, status.Error(codes.InvalidArgument, "tier1/binding: nil request")
	}
	// adapter 入力に変換。
	areq := dapr.BindingRequest{
		// Dapr Component 名。
		Name: req.GetName(),
		// 操作種別。
		Operation: req.GetOperation(),
		// データ本文。
		Data: req.GetData(),
		// メタデータ。
		Metadata: req.GetMetadata(),
		// テナント。
		TenantID: tenantIDOf(req.GetContext()),
	}
	// adapter 呼出。
	aresp, err := h.deps.BindingAdapter.Invoke(ctx, areq)
	// エラー翻訳。
	if err != nil {
		// 翻訳メッセージ。
		if isNotWired(err) {
			// Unimplemented 返却。
			return nil, status.Error(codes.Unimplemented, "tier1/binding: Invoke not yet wired to Dapr backend (plan 04-12)")
		}
		// Internal 返却。
		return nil, status.Errorf(codes.Internal, "tier1/binding: Invoke adapter error: %v", err)
	}
	// 応答返却。
	return &bindingv1.InvokeBindingResponse{
		// 応答本文。
		Data: aresp.Data,
		// メタデータ。
		Metadata: aresp.Metadata,
	}, nil
}
