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
//   2 系統の adapter 越しに実装する。adapter 未注入時は Unimplemented を返す。
//
// 短期 vs 長期 の振り分け（FR-T1-WORKFLOW-001）:
//   StartRequest.backend で hint を受け取り、handler がルーティングする。
//   - BACKEND_TEMPORAL: Temporal adapter（長期実行・高機能、cron / continue-as-new）
//   - BACKEND_DAPR:     Dapr Workflow adapter（短期実行・上限 7 日）
//   - BACKEND_AUTO:     handler 既定の routing ルールに従う（現状: Temporal にフォールバック）
//   選択結果は StartResponse.backend に返す。後続 Signal / Query / Cancel / Terminate /
//   GetStatus は workflow_id を見て同じ backend に dispatch する（routing table 参照）。

// Package workflow は t1-workflow Pod が登録する WorkflowService の handler を提供する。
package workflow

import (
	"context"
	"sync"

	// Dapr Workflow adapter（FR-T1-WORKFLOW-001 短期向け）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/daprwf"
	// Temporal adapter（長期向け）。
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
	// Temporal adapter（長期向け、nil 時は当該 backend RPC で Unimplemented）。
	WorkflowAdapter temporal.WorkflowAdapter
	// Dapr Workflow adapter（短期向け、nil 時は当該 backend RPC で Unimplemented）。
	DaprAdapter daprwf.WorkflowAdapter
}

// pickBackend は StartRequest.backend を解決する。AUTO は Temporal にフォールバック。
// （長期 default の方が安全側。短期は明示 BACKEND_DAPR を要求する運用とする。）
func pickBackend(hint workflowv1.WorkflowBackend) workflowv1.WorkflowBackend {
	switch hint {
	case workflowv1.WorkflowBackend_BACKEND_DAPR:
		return workflowv1.WorkflowBackend_BACKEND_DAPR
	case workflowv1.WorkflowBackend_BACKEND_TEMPORAL:
		return workflowv1.WorkflowBackend_BACKEND_TEMPORAL
	default:
		return workflowv1.WorkflowBackend_BACKEND_TEMPORAL
	}
}

// workflowHandler は WorkflowService の handler 実装。
// route table（workflow_id → backend）を保持し、Start で振り分けた backend を
// 後続 RPC（Signal / Query / Cancel / Terminate / GetStatus）が引き継ぐ。
type workflowHandler struct {
	workflowv1.UnimplementedWorkflowServiceServer
	deps Deps
	// 排他制御。
	routesMu sync.RWMutex
	// workflow_id → backend の対応表（Start 成功時に書き込み）。
	routes map[string]workflowv1.WorkflowBackend
}

// rememberRoute は workflow_id を backend に紐付けて記録する。
func (h *workflowHandler) rememberRoute(workflowID string, backend workflowv1.WorkflowBackend) {
	if workflowID == "" {
		return
	}
	h.routesMu.Lock()
	defer h.routesMu.Unlock()
	if h.routes == nil {
		h.routes = map[string]workflowv1.WorkflowBackend{}
	}
	h.routes[workflowID] = backend
}

// resolveRoute は workflow_id → backend を引く。未登録なら Temporal を default で返す。
func (h *workflowHandler) resolveRoute(workflowID string) workflowv1.WorkflowBackend {
	h.routesMu.RLock()
	defer h.routesMu.RUnlock()
	if b, ok := h.routes[workflowID]; ok {
		return b
	}
	return workflowv1.WorkflowBackend_BACKEND_TEMPORAL
}

// Register は WorkflowService を gRPC server に登録する hook を返す。
func Register(deps Deps) func(*grpc.Server) {
	return func(srv *grpc.Server) {
		workflowv1.RegisterWorkflowServiceServer(srv, &workflowHandler{
			deps:   deps,
			routes: map[string]workflowv1.WorkflowBackend{},
		})
	}
}

// translateErr は backend SDK のエラーを gRPC status code に翻訳する。
// 本層では Internal 一律で十分（NotFound / AlreadyExists 翻訳は plan 04-15）。
func translateErr(err error, rpc string) error {
	return status.Errorf(codes.Internal, "tier1/workflow: %s: %v", rpc, err)
}

// notWiredBackend は backend ごとの未注入応答。
func notWiredBackend(rpc string, backend workflowv1.WorkflowBackend) error {
	switch backend {
	case workflowv1.WorkflowBackend_BACKEND_DAPR:
		return status.Errorf(codes.Unimplemented, "tier1/workflow: %s not yet wired to Dapr Workflow", rpc)
	default:
		return status.Errorf(codes.Unimplemented, "tier1/workflow: %s not yet wired to Temporal", rpc)
	}
}

// Start はワークフロー開始。backend hint を解決し対応する adapter に委譲する。
// 選択された backend は StartResponse.backend に返却する。同 workflow_id への後続
// Signal / Query / Cancel / Terminate / GetStatus は本 routes 表で同 backend に
// 振り分けられる（呼び出し元は backend を再指定する必要なし）。
func (h *workflowHandler) Start(ctx context.Context, req *workflowv1.StartRequest) (*workflowv1.StartResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "tier1/workflow: nil request")
	}
	// backend hint を解決する（AUTO は Temporal にフォールバック）。
	backend := pickBackend(req.GetBackend())
	switch backend {
	case workflowv1.WorkflowBackend_BACKEND_DAPR:
		// Dapr Workflow 経路。
		if h.deps.DaprAdapter == nil {
			return nil, notWiredBackend("Start", backend)
		}
		resp, err := h.deps.DaprAdapter.Start(ctx, daprwf.StartRequest{
			WorkflowType: req.GetWorkflowType(),
			WorkflowID:   req.GetWorkflowId(),
			Input:        req.GetInput(),
			Idempotent:   req.GetIdempotent(),
			TenantID:     req.GetContext().GetTenantId(),
		})
		if err != nil {
			return nil, translateErr(err, "Start")
		}
		// 後続 RPC が同 backend に dispatch されるよう routes 表へ登録する。
		h.rememberRoute(resp.WorkflowID, backend)
		return &workflowv1.StartResponse{
			WorkflowId: resp.WorkflowID,
			RunId:      resp.RunID,
			Backend:    backend,
		}, nil
	default:
		// Temporal 経路（BACKEND_TEMPORAL / BACKEND_AUTO は両方ここに来る）。
		if h.deps.WorkflowAdapter == nil {
			return nil, notWiredBackend("Start", backend)
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
		h.rememberRoute(resp.WorkflowID, backend)
		return &workflowv1.StartResponse{
			WorkflowId: resp.WorkflowID,
			RunId:      resp.RunID,
			Backend:    backend,
		}, nil
	}
}

// Signal はシグナル送信。Start で記録した routes 表に従い backend を選ぶ。
func (h *workflowHandler) Signal(ctx context.Context, req *workflowv1.SignalRequest) (*workflowv1.SignalResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "tier1/workflow: nil request")
	}
	backend := h.resolveRoute(req.GetWorkflowId())
	if backend == workflowv1.WorkflowBackend_BACKEND_DAPR {
		if h.deps.DaprAdapter == nil {
			return nil, notWiredBackend("Signal", backend)
		}
		if err := h.deps.DaprAdapter.Signal(ctx, daprwf.SignalRequest{
			WorkflowID: req.GetWorkflowId(),
			SignalName: req.GetSignalName(),
			Payload:    req.GetPayload(),
		}); err != nil {
			return nil, translateErr(err, "Signal")
		}
		return &workflowv1.SignalResponse{}, nil
	}
	if h.deps.WorkflowAdapter == nil {
		return nil, notWiredBackend("Signal", backend)
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
	backend := h.resolveRoute(req.GetWorkflowId())
	if backend == workflowv1.WorkflowBackend_BACKEND_DAPR {
		if h.deps.DaprAdapter == nil {
			return nil, notWiredBackend("Query", backend)
		}
		resp, err := h.deps.DaprAdapter.Query(ctx, daprwf.QueryRequest{
			WorkflowID: req.GetWorkflowId(),
			QueryName:  req.GetQueryName(),
			Payload:    req.GetPayload(),
		})
		if err != nil {
			return nil, translateErr(err, "Query")
		}
		return &workflowv1.QueryResponse{Result: resp.Result}, nil
	}
	if h.deps.WorkflowAdapter == nil {
		return nil, notWiredBackend("Query", backend)
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
	backend := h.resolveRoute(req.GetWorkflowId())
	if backend == workflowv1.WorkflowBackend_BACKEND_DAPR {
		if h.deps.DaprAdapter == nil {
			return nil, notWiredBackend("Cancel", backend)
		}
		if err := h.deps.DaprAdapter.Cancel(ctx, daprwf.CancelRequest{
			WorkflowID: req.GetWorkflowId(),
			Reason:     req.GetReason(),
		}); err != nil {
			return nil, translateErr(err, "Cancel")
		}
		return &workflowv1.CancelResponse{}, nil
	}
	if h.deps.WorkflowAdapter == nil {
		return nil, notWiredBackend("Cancel", backend)
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
	backend := h.resolveRoute(req.GetWorkflowId())
	if backend == workflowv1.WorkflowBackend_BACKEND_DAPR {
		if h.deps.DaprAdapter == nil {
			return nil, notWiredBackend("Terminate", backend)
		}
		if err := h.deps.DaprAdapter.Terminate(ctx, daprwf.TerminateRequest{
			WorkflowID: req.GetWorkflowId(),
			Reason:     req.GetReason(),
		}); err != nil {
			return nil, translateErr(err, "Terminate")
		}
		return &workflowv1.TerminateResponse{}, nil
	}
	if h.deps.WorkflowAdapter == nil {
		return nil, notWiredBackend("Terminate", backend)
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
	backend := h.resolveRoute(req.GetWorkflowId())
	if backend == workflowv1.WorkflowBackend_BACKEND_DAPR {
		if h.deps.DaprAdapter == nil {
			return nil, notWiredBackend("GetStatus", backend)
		}
		resp, err := h.deps.DaprAdapter.GetStatus(ctx, daprwf.GetStatusRequest{
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
	if h.deps.WorkflowAdapter == nil {
		return nil, notWiredBackend("GetStatus", backend)
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
