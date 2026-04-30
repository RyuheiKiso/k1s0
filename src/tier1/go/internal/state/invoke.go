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
	// NFR-E-AC-003: tenant_id 越境防止のため必須検証。
	tid, err := requireTenantID(req.GetContext(), "Invoke.Invoke")
	if err != nil {
		return nil, err
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
		TenantID: tid,
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

// chunkSize は InvokeStream で応答 bytes を分割するときのデフォルトチャンクサイズ（4 KiB）。
// gRPC のフレーム上限 (default 4 MiB) よりはるかに小さく、レイテンシよりスループット優先。
const invokeStreamChunkSize = 4 * 1024

// InvokeStream は server-streaming RPC。Dapr SDK の InvokeMethod は完全な
// streaming を直接公開しないため、まず adapter.Invoke で全 bytes を取得し、
// それをチャンク分割して stream.Send する。proto 契約（stream InvokeChunk + eof
// フラグ）を満たす最小実装。upstream が真の streaming に対応した時点で本実装を
// 直接 streaming proxy に置き換える（adapter interface 不変）。
func (h *invokeHandler) InvokeStream(req *serviceinvokev1.InvokeRequest, stream serviceinvokev1.InvokeService_InvokeStreamServer) error {
	if req == nil {
		return status.Error(codes.InvalidArgument, "tier1/serviceinvoke: nil request")
	}
	if h.deps.InvokeAdapter == nil {
		return status.Error(codes.Unimplemented, "tier1/serviceinvoke: InvokeStream not yet wired to Dapr backend")
	}
	// NFR-E-AC-003: tenant_id 越境防止のため必須検証。
	tid, terr := requireTenantID(req.GetContext(), "Invoke.InvokeStream")
	if terr != nil {
		return terr
	}
	areq := dapr.InvokeRequest{
		AppID:       req.GetAppId(),
		Method:      req.GetMethod(),
		Data:        req.GetData(),
		ContentType: req.GetContentType(),
		TenantID:    tid,
		TimeoutMs:   req.GetTimeoutMs(),
	}
	aresp, err := h.deps.InvokeAdapter.Invoke(stream.Context(), areq)
	if err != nil {
		return translateInvokeErr(err, "InvokeStream")
	}
	body := aresp.Data
	// 本文が空なら eof=true の単一チャンクを 1 件だけ送る（proto 契約に沿う）。
	if len(body) == 0 {
		return stream.Send(&serviceinvokev1.InvokeChunk{Eof: true})
	}
	for offset := 0; offset < len(body); offset += invokeStreamChunkSize {
		end := offset + invokeStreamChunkSize
		if end > len(body) {
			end = len(body)
		}
		eof := end == len(body)
		if err := stream.Send(&serviceinvokev1.InvokeChunk{
			Data: body[offset:end],
			Eof:  eof,
		}); err != nil {
			return status.Errorf(codes.Internal, "tier1/serviceinvoke: stream.Send: %v", err)
		}
	}
	return nil
}

// translateInvokeErr は ServiceInvoke 用のエラー翻訳。
func translateInvokeErr(err error, rpc string) error {
	// ErrNotWired は Unimplemented に翻訳する。
	if isNotWired(err) {
		// メッセージに RPC 名と plan ID を含める。
		return status.Errorf(codes.Unimplemented, "tier1/serviceinvoke: %s not yet wired to Dapr backend (plan 04-11)", rpc)
	}
	// dapr が返す gRPC status を尊重する。serviceinvoke は対象 service が gRPC で
	// status を返すケース（NotFound / PermissionDenied / Unavailable 等）が多く、
	// それらを Internal に潰すと client は適切な再試行判定ができない。
	if st, ok := status.FromError(err); ok && st.Code() != codes.Unknown && st.Code() != codes.OK {
		return status.Errorf(st.Code(), "tier1/serviceinvoke: %s adapter error: %s", rpc, st.Message())
	}
	// 想定外エラーは Internal。
	return status.Errorf(codes.Internal, "tier1/serviceinvoke: %s adapter error: %v", rpc, err)
}
