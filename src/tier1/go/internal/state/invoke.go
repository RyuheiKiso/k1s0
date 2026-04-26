// 本ファイルは t1-state Pod の InvokeService 2 RPC ハンドラ実装。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/01_Service_Invoke_API.md
//
// scope（リリース時点 placeholder）: adapter は ErrNotWired を返却。実 Dapr Service Invocation
// 結線は plan 04-11。

package state

// 標準 / 内部パッケージ。
import (
	// context 伝搬。
	"context"
	// Dapr adapter。
	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
	// SDK 生成 stub の InvokeService 型。
	serviceinvokev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/serviceinvoke/v1"
	// gRPC エラーコード。
	"google.golang.org/grpc/codes"
	// gRPC ステータスエラー。
	"google.golang.org/grpc/status"
)

// invokeHandler は InvokeService の handler 実装。
type invokeHandler struct {
	// 将来 RPC 用の埋め込み。
	serviceinvokev1.UnimplementedInvokeServiceServer
	// adapter 集合への参照。
	deps Deps
}

// Invoke は任意サービスの任意メソッド呼出。
func (h *invokeHandler) Invoke(ctx context.Context, req *serviceinvokev1.InvokeRequest) (*serviceinvokev1.InvokeResponse, error) {
	// 入力 nil 防御。
	if req == nil {
		// 不正引数返却。
		return nil, status.Error(codes.InvalidArgument, "tier1/serviceinvoke: nil request")
	}
	// adapter 入力に変換。
	areq := dapr.InvokeRequest{
		// 呼出先アプリ識別子。
		AppID: req.GetAppId(),
		// メソッド名。
		Method: req.GetMethod(),
		// データ本文。
		Data: req.GetData(),
		// Content-Type。
		ContentType: req.GetContentType(),
		// テナント。
		TenantID: tenantIDOf(req.GetContext()),
		// タイムアウト。
		TimeoutMs: req.GetTimeoutMs(),
	}
	// adapter 呼出。
	aresp, err := h.deps.InvokeAdapter.Invoke(ctx, areq)
	// エラー翻訳。
	if err != nil {
		// 翻訳 helper（state.go 定義）を invoke 用にカスタマイズ。
		return nil, translateInvokeErr(err, "Invoke")
	}
	// proto 応答に変換して返却する。
	return &serviceinvokev1.InvokeResponse{
		// 応答本文。
		Data: aresp.Data,
		// Content-Type。
		ContentType: aresp.ContentType,
		// HTTP ステータス相当。
		Status: aresp.Status,
	}, nil
}

// InvokeStream はストリーミング呼出。本リリース時点 では Unimplemented。
func (h *invokeHandler) InvokeStream(_ *serviceinvokev1.InvokeRequest, _ serviceinvokev1.InvokeService_InvokeStreamServer) error {
	// stream は plan 04-11 で実装。
	return status.Error(codes.Unimplemented, "tier1/serviceinvoke: InvokeStream not yet wired (plan 04-11)")
}

// translateInvokeErr は ServiceInvoke 用のエラー翻訳。
func translateInvokeErr(err error, rpc string) error {
	// ErrNotWired は Unimplemented に翻訳する。
	if isNotWired(err) {
		// メッセージに RPC 名と plan ID を含める。
		return status.Errorf(codes.Unimplemented, "tier1/serviceinvoke: %s not yet wired to Dapr backend (plan 04-11)", rpc)
	}
	// 想定外エラーは Internal。
	return status.Errorf(codes.Internal, "tier1/serviceinvoke: %s adapter error: %v", rpc, err)
}
