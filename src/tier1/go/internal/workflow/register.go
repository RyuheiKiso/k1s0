// 本ファイルは t1-workflow Pod が gRPC server に登録する WorkflowService の handler。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md
//     - DS-SW-COMP-010（t1-workflow: Dapr Workflow / Temporal pluggable、固定 3 replica、HPA 禁止）
//   docs/02_構想設計/adr/ADR-RULE-002-temporal.md
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/06_Workflow_API.md
//
// 役割（plan 04-07 結線済 / Temporal 経路）:
//   WorkflowService の 6 RPC（Start / Signal / Query / Cancel / Terminate / GetStatus）を
//   Temporal adapter 越しに実装する。adapter 未注入時は Unimplemented を返す。
//
// 短期 vs 長期 の振り分け:
//   現状は Temporal 経路のみ実装。Dapr Workflow（短期）への振り分けは plan 04-14 で
//   ポリシー層を本 handler の上に追加して実現する。

// Package workflow は t1-workflow Pod が登録する WorkflowService の handler を提供する。
package workflow

import (
	"context"

	// Temporal adapter（本 Pod 専用）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/temporal"
	// SDK 生成 stub の WorkflowService 型。
	workflowv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/workflow/v1"
	// gRPC server 型。
	"google.golang.org/grpc"
	// gRPC エラーコード。
	"google.golang.org/grpc/codes"
	// gRPC ステータスエラー。
	"google.golang.org/grpc/status"
)

// Deps は WorkflowService handler が依存する adapter 集合。
type Deps struct {
	// Temporal adapter（nil 時は全 RPC で Unimplemented を返す）。
	WorkflowAdapter temporal.WorkflowAdapter
}

// workflowHandler は WorkflowService の handler 実装。
type workflowHandler struct {
	workflowv1.UnimplementedWorkflowServiceServer
	deps Deps
}

// Register は WorkflowService を gRPC server に登録する hook を返す。
func Register(deps Deps) func(*grpc.Server) {
	return func(srv *grpc.Server) {
		workflowv1.RegisterWorkflowServiceServer(srv, &workflowHandler{deps: deps})
	}
}

// translateErr は Temporal SDK のエラーを gRPC status code に翻訳する。
// Temporal は serviceerror.NotFound 等を返すため、本層では Internal 一律で十分。
// 将来的には NotFound / AlreadyExists / FailedPrecondition への翻訳を追加予定。
func translateErr(err error, rpc string) error {
	return status.Errorf(codes.Internal, "tier1/workflow: %s: %v", rpc, err)
}

// notWired は adapter 未注入時の標準応答。
func notWired(rpc string) error {
	return status.Errorf(codes.Unimplemented, "tier1/workflow: %s not yet wired to Temporal", rpc)
}

// Start はワークフロー開始。
func (h *workflowHandler) Start(ctx context.Context, req *workflowv1.StartRequest) (*workflowv1.StartResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "tier1/workflow: nil request")
	}
	if h.deps.WorkflowAdapter == nil {
		return nil, notWired("Start")
	}
	resp, err := h.deps.WorkflowAdapter.Start(ctx, temporal.StartRequest{
		WorkflowType: req.GetWorkflowType(),
		WorkflowID:   req.GetWorkflowId(),
		Input:        req.GetInput(),
		Idempotent:   req.GetIdempotent(),
		TenantID:     req.GetContext().GetTenantId(),
	})
	if err != nil {
		return nil, translateErr(err, "Start")
	}
	return &workflowv1.StartResponse{
		WorkflowId: resp.WorkflowID,
		RunId:      resp.RunID,
	}, nil
}

// Signal はシグナル送信。
func (h *workflowHandler) Signal(ctx context.Context, req *workflowv1.SignalRequest) (*workflowv1.SignalResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "tier1/workflow: nil request")
	}
	if h.deps.WorkflowAdapter == nil {
		return nil, notWired("Signal")
	}
	if err := h.deps.WorkflowAdapter.Signal(ctx, temporal.SignalRequest{
		WorkflowID: req.GetWorkflowId(),
		SignalName: req.GetSignalName(),
		Payload:    req.GetPayload(),
	}); err != nil {
		return nil, translateErr(err, "Signal")
	}
	return &workflowv1.SignalResponse{}, nil
}

// Query はワークフロー状態のクエリ。
func (h *workflowHandler) Query(ctx context.Context, req *workflowv1.QueryRequest) (*workflowv1.QueryResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "tier1/workflow: nil request")
	}
	if h.deps.WorkflowAdapter == nil {
		return nil, notWired("Query")
	}
	resp, err := h.deps.WorkflowAdapter.Query(ctx, temporal.QueryRequest{
		WorkflowID: req.GetWorkflowId(),
		QueryName:  req.GetQueryName(),
		Payload:    req.GetPayload(),
	})
	if err != nil {
		return nil, translateErr(err, "Query")
	}
	return &workflowv1.QueryResponse{Result: resp.Result}, nil
}

// Cancel はワークフローのキャンセル。
func (h *workflowHandler) Cancel(ctx context.Context, req *workflowv1.CancelRequest) (*workflowv1.CancelResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "tier1/workflow: nil request")
	}
	if h.deps.WorkflowAdapter == nil {
		return nil, notWired("Cancel")
	}
	if err := h.deps.WorkflowAdapter.Cancel(ctx, temporal.CancelRequest{
		WorkflowID: req.GetWorkflowId(),
		Reason:     req.GetReason(),
	}); err != nil {
		return nil, translateErr(err, "Cancel")
	}
	return &workflowv1.CancelResponse{}, nil
}

// Terminate はワークフローの強制終了。
func (h *workflowHandler) Terminate(ctx context.Context, req *workflowv1.TerminateRequest) (*workflowv1.TerminateResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "tier1/workflow: nil request")
	}
	if h.deps.WorkflowAdapter == nil {
		return nil, notWired("Terminate")
	}
	if err := h.deps.WorkflowAdapter.Terminate(ctx, temporal.TerminateRequest{
		WorkflowID: req.GetWorkflowId(),
		Reason:     req.GetReason(),
	}); err != nil {
		return nil, translateErr(err, "Terminate")
	}
	return &workflowv1.TerminateResponse{}, nil
}

// GetStatus はワークフローの状態取得。
func (h *workflowHandler) GetStatus(ctx context.Context, req *workflowv1.GetStatusRequest) (*workflowv1.GetStatusResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "tier1/workflow: nil request")
	}
	if h.deps.WorkflowAdapter == nil {
		return nil, notWired("GetStatus")
	}
	resp, err := h.deps.WorkflowAdapter.GetStatus(ctx, temporal.GetStatusRequest{
		WorkflowID: req.GetWorkflowId(),
	})
	if err != nil {
		return nil, translateErr(err, "GetStatus")
	}
	return &workflowv1.GetStatusResponse{
		Status: workflowv1.WorkflowStatus(resp.Status),
		RunId:  resp.RunID,
	}, nil
}
