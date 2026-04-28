// 本ファイルは Dapr Workflow building block の adapter エントリ。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/06_Workflow_API.md
//     - FR-T1-WORKFLOW-001（"短期は Dapr Workflow、長期実行は Temporal"）
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md
//     - DS-SW-COMP-010（t1-workflow Pod、Dapr Workflow / Temporal pluggable）
//
// 役割:
//   Temporal とは別系統で「短期ワークフロー」を Dapr Workflow building block 経由で
//   駆動する adapter を提供する。本 ファイルでは interface（Workflow{Adapter|Request|...}）と
//   in-memory backend を提供し、production（Dapr 結線）の実装は plan 04-14 で
//   `daprWorkflowClient` narrow interface に SDK を流し込む。
//
// 重要: tier1 の handler 層はこの interface に依存する。テストでは inMemoryDaprWorkflow
//   を注入する。production では SDK 結線済の adapter を注入することで、handler の
//   コードを書き換えずに backend を切替えられる。

// Package daprwf は Dapr Workflow を呼ぶための adapter 層。
package daprwf

import (
	"context"
	"errors"
)

// ErrNotWired は Dapr backend と未結線である旨を示すセンチネル。
// production で SDK 結線が無いまま起動された場合の handler 翻訳元。
var ErrNotWired = errors.New("tier1: Dapr Workflow backend not yet wired")

// ErrNotFound は workflow 未存在を表すセンチネル。
var ErrNotFound = errors.New("tier1: dapr workflow not found")

// 共通 status enum（Temporal adapter 側と意味的に揃える）。
type WorkflowStatusValue int

const (
	// 実行中。
	StatusRunning WorkflowStatusValue = 0
	// 正常完了。
	StatusCompleted WorkflowStatusValue = 1
	// 失敗終了。
	StatusFailed WorkflowStatusValue = 2
	// Cancel 済。
	StatusCanceled WorkflowStatusValue = 3
	// Terminate 済。
	StatusTerminated WorkflowStatusValue = 4
)

// StartRequest は Start 操作の入力。
type StartRequest struct {
	WorkflowType string
	WorkflowID   string
	Input        []byte
	Idempotent   bool
	TenantID     string
}

// StartResponse は Start 操作の応答。
type StartResponse struct {
	WorkflowID string
	RunID      string
}

// SignalRequest は Signal 操作の入力。
type SignalRequest struct {
	WorkflowID string
	SignalName string
	Payload    []byte
	TenantID   string
}

// QueryRequest は Query 操作の入力。
type QueryRequest struct {
	WorkflowID string
	QueryName  string
	Payload    []byte
	TenantID   string
}

// QueryResponse は Query 応答。
type QueryResponse struct {
	Result []byte
}

// CancelRequest は Cancel 操作の入力。
type CancelRequest struct {
	WorkflowID string
	Reason     string
	TenantID   string
}

// TerminateRequest は Terminate 操作の入力。
type TerminateRequest struct {
	WorkflowID string
	Reason     string
	TenantID   string
}

// GetStatusRequest は GetStatus 操作の入力。
type GetStatusRequest struct {
	WorkflowID string
	TenantID   string
}

// GetStatusResponse は GetStatus 応答。
type GetStatusResponse struct {
	Status WorkflowStatusValue
	RunID  string
}

// WorkflowAdapter は Dapr Workflow building block の操作集合。
// Temporal の WorkflowAdapter と意味論的に揃え、handler 側で routing するだけで
// 入れ替え可能にする。
type WorkflowAdapter interface {
	Start(ctx context.Context, req StartRequest) (StartResponse, error)
	Signal(ctx context.Context, req SignalRequest) error
	Query(ctx context.Context, req QueryRequest) (QueryResponse, error)
	Cancel(ctx context.Context, req CancelRequest) error
	Terminate(ctx context.Context, req TerminateRequest) error
	GetStatus(ctx context.Context, req GetStatusRequest) (GetStatusResponse, error)
}
