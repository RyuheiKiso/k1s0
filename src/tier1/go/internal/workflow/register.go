// 本ファイルは t1-workflow Pod が gRPC server に登録する WorkflowService の handler。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md
//     - DS-SW-COMP-010（t1-workflow: Dapr Workflow / Temporal pluggable、固定 3 replica、HPA 禁止）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/06_Workflow_API.md
//
// 役割（リリース時点 最小骨格）:
//   WorkflowService の 6 RPC（Start / Signal / Query / Cancel / Terminate / GetStatus）を登録する。
//   短期 workflow（Dapr Workflow + Valkey）と長期 workflow（Temporal + Postgres）の振り分けは
//   U-WORKFLOW-001 の決定 + plan 04-07 / 04-14 で実装。本リリース時点 は全 RPC で Unimplemented。
//
// 注: 本リリース時点 では internal/adapter/temporal/ および Dapr Workflow adapter は未配置。
//     plan 04-07（最小実装）/ plan 04-14（YAML 振り分け）で順次追加する。

// Package workflow は t1-workflow Pod が登録する WorkflowService の handler を提供する。
package workflow

// 標準 / 内部パッケージ。
import (
	// context 伝搬。
	"context"
	// SDK 生成 stub の WorkflowService 型。
	workflowv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/workflow/v1"
	// gRPC server 型。
	"google.golang.org/grpc"
	// gRPC エラーコード。
	"google.golang.org/grpc/codes"
	// gRPC ステータスエラー。
	"google.golang.org/grpc/status"
)

// workflowHandler は WorkflowService の handler 実装。
type workflowHandler struct {
	// 将来 RPC 用埋め込み。
	workflowv1.UnimplementedWorkflowServiceServer
}

// Register は WorkflowService を gRPC server に登録する hook を返す。
func Register() func(*grpc.Server) {
	// closure で handler を捕捉する。
	return func(srv *grpc.Server) {
		// WorkflowService を登録する（FR-T1-WORKFLOW-001〜005）。
		workflowv1.RegisterWorkflowServiceServer(srv, &workflowHandler{})
	}
}

// unimpl は workflow Pod 共通の Unimplemented エラー生成 helper。
// 短期/長期の振り分け根拠（plan 04-07 / 04-14）を全 RPC で揃える。
func unimpl(rpc string) error {
	// codes.Unimplemented + plan ID 付きメッセージを返却する。
	return status.Errorf(codes.Unimplemented, "tier1/workflow: %s not yet wired (plan 04-07 / 04-14, U-WORKFLOW-001 待ち)", rpc)
}

// Start はワークフロー開始。
func (h *workflowHandler) Start(_ context.Context, _ *workflowv1.StartRequest) (*workflowv1.StartResponse, error) {
	// 統一 helper 経由で返却する。
	return nil, unimpl("Start")
}

// Signal はシグナル送信。
func (h *workflowHandler) Signal(_ context.Context, _ *workflowv1.SignalRequest) (*workflowv1.SignalResponse, error) {
	// 統一 helper 経由。
	return nil, unimpl("Signal")
}

// Query はワークフロー状態のクエリ。
func (h *workflowHandler) Query(_ context.Context, _ *workflowv1.QueryRequest) (*workflowv1.QueryResponse, error) {
	// 統一 helper 経由。
	return nil, unimpl("Query")
}

// Cancel はワークフローのキャンセル。
func (h *workflowHandler) Cancel(_ context.Context, _ *workflowv1.CancelRequest) (*workflowv1.CancelResponse, error) {
	// 統一 helper 経由。
	return nil, unimpl("Cancel")
}

// Terminate はワークフローの強制終了。
func (h *workflowHandler) Terminate(_ context.Context, _ *workflowv1.TerminateRequest) (*workflowv1.TerminateResponse, error) {
	// 統一 helper 経由。
	return nil, unimpl("Terminate")
}

// GetStatus はワークフローの状態取得。
func (h *workflowHandler) GetStatus(_ context.Context, _ *workflowv1.GetStatusRequest) (*workflowv1.GetStatusResponse, error) {
	// 統一 helper 経由。
	return nil, unimpl("GetStatus")
}
