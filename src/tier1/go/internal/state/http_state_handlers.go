// 本ファイルは State API の HTTP/JSON gateway 用 RPC ハンドラ adapter。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/00_tier1_API共通規約.md §「HTTP/JSON 互換」
//
// 役割:
//   common.HTTPGateway.RegisterStateRoutes に渡す common.StateRPCHandlers を組み立てる。
//   各 RPC は in-process で内部 stateHandler を直接呼ぶ。
//   protojson Unmarshal → handler 呼出 → proto Response を返す（gateway 側で再度 protojson Marshal）。
//
//   gateway は内部で gRPC metadata（authorization / traceparent / idempotency-key）を
//   ctx に attach する。本 adapter で gRPC interceptor chain を改めて適用するか、もしくは
//   gateway を gRPC server の前段として使うかは cmd 側の設定次第。最小実装として、本 adapter は
//   common.UnmarshalJSON で型安全な request 復元のみを担い、interceptor chain との結合は
//   cmd/state/main.go で行う（gRPC server を内部 client として使えば全 interceptor が走る）。

package state

import (
	"context"
	"errors"

	"github.com/k1s0/k1s0/src/tier1/go/internal/common"
	statev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/state/v1"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/proto"
)

// MakeHTTPHandlers は HTTP/JSON gateway 用 handler 集合を組み立てる。
// stateSvc は in-process StateServiceServer 実装（Register hook が登録するもの）。
// 内部で protojson 解釈 → 直接呼出（gRPC を経由しない、軽量経路）。
//
// 注意:
//   本 adapter は AuthInterceptor / RateLimit / Audit の chain を経由しない。
//   gateway 経由でも認証・レート制限・監査が必要な場合は、cmd 側で grpc.ServerInProc /
//   grpc.WithDefaultCallOptions で interceptor chain を共有する設計にする。
//   release-initial の最小実装としては interceptor 経由（gRPC 経路）を使う運用で OK。
func MakeHTTPHandlers(stateSvc statev1.StateServiceServer) common.StateRPCHandlers {
	return common.StateRPCHandlers{
		Get: func(ctx context.Context, body []byte) (proto.Message, error) {
			req := &statev1.GetRequest{}
			if err := common.UnmarshalJSON(body, req); err != nil {
				return nil, err
			}
			if stateSvc == nil {
				return nil, errors.New("state service not wired")
			}
			return stateSvc.Get(ctx, req)
		},
		Set: func(ctx context.Context, body []byte) (proto.Message, error) {
			req := &statev1.SetRequest{}
			if err := common.UnmarshalJSON(body, req); err != nil {
				return nil, err
			}
			if stateSvc == nil {
				return nil, status.Error(codes.Unavailable, "state service not wired")
			}
			return stateSvc.Set(ctx, req)
		},
		Delete: func(ctx context.Context, body []byte) (proto.Message, error) {
			req := &statev1.DeleteRequest{}
			if err := common.UnmarshalJSON(body, req); err != nil {
				return nil, err
			}
			if stateSvc == nil {
				return nil, status.Error(codes.Unavailable, "state service not wired")
			}
			return stateSvc.Delete(ctx, req)
		},
	}
}

// NewStateServiceServer は HTTP gateway / in-process 統合テスト用に
// stateHandler を直接生成する exported helper。
func NewStateServiceServer(deps Deps) statev1.StateServiceServer {
	return &stateHandler{deps: deps}
}
