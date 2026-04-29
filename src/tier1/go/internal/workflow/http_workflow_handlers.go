// 本ファイルは Workflow API の HTTP/JSON gateway 用 RPC ハンドラ adapter。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/00_tier1_API共通規約.md §「HTTP/JSON 互換」
//
// 役割:
//   common.HTTPGateway.RegisterWorkflowRoutes に渡す common.WorkflowRPCHandlers を組み立てる。
//   Start / Signal / Query / Cancel / Terminate / GetStatus の 6 RPC を protojson Unmarshal
//   経由で in-process WorkflowServiceServer に dispatch する（全 RPC unary）。

package workflow

import (
	"context"

	"github.com/k1s0/k1s0/src/tier1/go/internal/common"
	workflowv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/workflow/v1"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/proto"
)

// MakeHTTPHandlers は HTTP/JSON gateway 用 Workflow handler 集合を組み立てる。
func MakeHTTPHandlers(svc workflowv1.WorkflowServiceServer) common.WorkflowRPCHandlers {
	notWired := func() error { return status.Error(codes.Unavailable, "workflow service not wired") }
	return common.WorkflowRPCHandlers{
		Start: func(ctx context.Context, body []byte) (proto.Message, error) {
			req := &workflowv1.StartRequest{}
			if err := common.UnmarshalJSON(body, req); err != nil {
				return nil, err
			}
			if svc == nil {
				return nil, notWired()
			}
			return svc.Start(ctx, req)
		},
		Signal: func(ctx context.Context, body []byte) (proto.Message, error) {
			req := &workflowv1.SignalRequest{}
			if err := common.UnmarshalJSON(body, req); err != nil {
				return nil, err
			}
			if svc == nil {
				return nil, notWired()
			}
			return svc.Signal(ctx, req)
		},
		Query: func(ctx context.Context, body []byte) (proto.Message, error) {
			req := &workflowv1.QueryRequest{}
			if err := common.UnmarshalJSON(body, req); err != nil {
				return nil, err
			}
			if svc == nil {
				return nil, notWired()
			}
			return svc.Query(ctx, req)
		},
		Cancel: func(ctx context.Context, body []byte) (proto.Message, error) {
			req := &workflowv1.CancelRequest{}
			if err := common.UnmarshalJSON(body, req); err != nil {
				return nil, err
			}
			if svc == nil {
				return nil, notWired()
			}
			return svc.Cancel(ctx, req)
		},
		Terminate: func(ctx context.Context, body []byte) (proto.Message, error) {
			req := &workflowv1.TerminateRequest{}
			if err := common.UnmarshalJSON(body, req); err != nil {
				return nil, err
			}
			if svc == nil {
				return nil, notWired()
			}
			return svc.Terminate(ctx, req)
		},
		GetStatus: func(ctx context.Context, body []byte) (proto.Message, error) {
			req := &workflowv1.GetStatusRequest{}
			if err := common.UnmarshalJSON(body, req); err != nil {
				return nil, err
			}
			if svc == nil {
				return nil, notWired()
			}
			return svc.GetStatus(ctx, req)
		},
	}
}

// NewWorkflowServiceServer は HTTP gateway / 統合テスト用に workflowHandler を直接生成する exported helper。
func NewWorkflowServiceServer(deps Deps) workflowv1.WorkflowServiceServer {
	return &workflowHandler{
		deps:   deps,
		routes: map[string]workflowv1.WorkflowBackend{},
	}
}
