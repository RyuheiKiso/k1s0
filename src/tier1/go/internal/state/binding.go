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
	// 共通 idempotency cache（共通規約 §「冪等性と再試行」）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/common"
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
// 共通規約 §「冪等性と再試行」: idempotency_key 指定時は外部送信（SMTP / S3 等）の
// 重複を防ぐため、同一キーの再試行で初回 InvokeBindingResponse を返す。
func (h *bindingHandler) Invoke(ctx context.Context, req *bindingv1.InvokeBindingRequest) (*bindingv1.InvokeBindingResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "tier1/binding: nil request")
	}
	// NFR-E-AC-003: tenant_id 越境防止のため必須検証。
	tid, err := requireTenantID(req.GetContext(), "Binding.Invoke")
	if err != nil {
		return nil, err
	}
	// 必須入力の事前検証（adapter 越しに dapr SDK が返す errors.New("...required") を
	// codes.Internal として上位に漏らさないよう、handler で InvalidArgument として弾く）。
	if req.GetName() == "" {
		return nil, status.Error(codes.InvalidArgument, "tier1/binding: name required")
	}
	if req.GetOperation() == "" {
		return nil, status.Error(codes.InvalidArgument, "tier1/binding: operation required")
	}
	// 実 Invoke 実行クロージャ。idempotency cache hit 時は呼ばれない。
	doInvoke := func() (interface{}, error) {
		areq := dapr.BindingRequest{
			Name:      req.GetName(),
			Operation: req.GetOperation(),
			Data:      req.GetData(),
			Metadata:  req.GetMetadata(),
			TenantID:  tid,
		}
		aresp, err := h.deps.BindingAdapter.Invoke(ctx, areq)
		if err != nil {
			if isNotWired(err) {
				return nil, status.Error(codes.Unimplemented, "tier1/binding: Invoke not yet wired to Dapr backend (plan 04-12)")
			}
			// dapr が返す gRPC status を尊重する（PermissionDenied / FailedPrecondition 等）
			if st, ok := status.FromError(err); ok && st.Code() != codes.Unknown && st.Code() != codes.OK {
				return nil, status.Errorf(st.Code(), "tier1/binding: Invoke adapter error: %s", st.Message())
			}
			return nil, status.Errorf(codes.Internal, "tier1/binding: Invoke adapter error: %v", err)
		}
		return &bindingv1.InvokeBindingResponse{
			Data:     aresp.Data,
			Metadata: aresp.Metadata,
		}, nil
	}
	idempKey := common.IdempotencyKey(tid, "Binding.Invoke", req.GetIdempotencyKey())
	if idempKey == "" || h.deps.Idempotency == nil {
		resp, err := doInvoke()
		if err != nil {
			return nil, err
		}
		return resp.(*bindingv1.InvokeBindingResponse), nil
	}
	resp, err := h.deps.Idempotency.GetOrCompute(ctx, idempKey, doInvoke)
	if err != nil {
		return nil, err
	}
	return resp.(*bindingv1.InvokeBindingResponse), nil
}
