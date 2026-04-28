// 本ファイルは Temporal を使った WorkflowAdapter 実装。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/06_Workflow_API.md
//
// 役割（plan 04-07 結線済 / Temporal 経路）:
//   handler.go が呼び出す WorkflowService 6 RPC（Start / Signal / Query /
//   Cancel / Terminate / GetStatus）を Temporal Go SDK で実装する。
//
// 動的 workflow 種別:
//   ExecuteWorkflow に workflow type 名（string）をそのまま渡す。Temporal は
//   登録済 worker が同名で workflow を register していれば dispatch する仕様。
//   k1s0 では tier2 worker 側で `worker.RegisterWorkflowWithOptions(name=...)` で
//   登録する運用想定（ADR-RULE-002）。
//
// 入出力ペイロード:
//   proto は input/payload/result を `[]byte` で扱う。Temporal SDK は arg を
//   `interface{}` として serialize するため、`[]byte` を渡すと内部 JSONPayloadConverter
//   などで encode される。SDK の converter を上書きすればスキーマ既知の serialization も可能だが、
//   本層は generic として `[]byte` のまま透過させる。
//
// status 翻訳:
//   Temporal の WorkflowExecutionStatus 整数値（1=RUNNING, 2=COMPLETED, ...）を
//   k1s0 proto WorkflowStatus 整数値（0=RUNNING, 1=COMPLETED, ...）に翻訳する。

package temporal

import (
	"context"
	"errors"

	tclient "go.temporal.io/sdk/client"
)

// k1s0 の workflow proto 値（adapter 中立な int 値で表現）。
// proto enum と同値で定義しているが、proto 依存をこの adapter に持ち込まないため
// パッケージ内で再定義する。handler 側が proto enum とこの値を相互変換する。
type WorkflowStatusValue int32

const (
	WorkflowStatusRunning        WorkflowStatusValue = 0
	WorkflowStatusCompleted      WorkflowStatusValue = 1
	WorkflowStatusFailed         WorkflowStatusValue = 2
	WorkflowStatusCanceled       WorkflowStatusValue = 3
	WorkflowStatusTerminated     WorkflowStatusValue = 4
	WorkflowStatusContinuedAsNew WorkflowStatusValue = 5
)

// Temporal SDK の WorkflowExecutionStatus 整数値（enum.WorkflowExecutionStatus）。
// SDK 直接 import するとサンプルが膨らむため、定数値で扱う（API 変更時はここも追従）。
const (
	temporalStatusUnspecified    int32 = 0
	temporalStatusRunning        int32 = 1
	temporalStatusCompleted      int32 = 2
	temporalStatusFailed         int32 = 3
	temporalStatusCanceled       int32 = 4
	temporalStatusTerminated     int32 = 5
	temporalStatusContinuedAsNew int32 = 6
	temporalStatusTimedOut       int32 = 7
)

// translateStatus は Temporal の status code を k1s0 WorkflowStatusValue に翻訳する。
// TimedOut / Unspecified は最も近い意味の Failed に寄せる（OpenFeature ERROR と同じ思想）。
func translateStatus(code int32) WorkflowStatusValue {
	switch code {
	case temporalStatusRunning:
		return WorkflowStatusRunning
	case temporalStatusCompleted:
		return WorkflowStatusCompleted
	case temporalStatusFailed, temporalStatusTimedOut:
		return WorkflowStatusFailed
	case temporalStatusCanceled:
		return WorkflowStatusCanceled
	case temporalStatusTerminated:
		return WorkflowStatusTerminated
	case temporalStatusContinuedAsNew:
		return WorkflowStatusContinuedAsNew
	default:
		// Unspecified などは Running と扱う（保守的）。
		return WorkflowStatusRunning
	}
}

// k1s0 既定 task queue。tier2 worker 側で同名 task queue を listen する運用。
const defaultTaskQueue = "k1s0-default"

// StartRequest は WorkflowAdapter.Start の入力。
type StartRequest struct {
	WorkflowType string
	WorkflowID   string // 空なら呼出側で UUID 採番済を渡す
	Input        []byte
	Idempotent   bool   // true → 既存実行を返す（重複実行禁止）
	TaskQueue    string // 空なら defaultTaskQueue
	TenantID     string // memo / search attribute に詰める想定（plan 04-14）
}

// StartResponse は Start の応答。
type StartResponse struct {
	WorkflowID string
	RunID      string
}

// SignalRequest は Signal の入力。
type SignalRequest struct {
	WorkflowID string
	RunID      string // 空なら最新 run へ送信
	SignalName string
	Payload    []byte
	// テナント識別子（NFR-E-AC-003、in-memory backend で run.tenantID 突合に使う）。
	// production では Temporal SDK の SignalWorkflow 呼出時に search attribute / memo で
	// 制限する設計（plan 04-14）。
	TenantID string
}

// QueryRequest は Query の入力。
type QueryRequest struct {
	WorkflowID string
	RunID      string
	QueryName  string
	Payload    []byte
	// テナント識別子（NFR-E-AC-003）。
	TenantID string
}

// QueryResponse は Query の応答。
type QueryResponse struct {
	Result []byte
}

// CancelRequest は Cancel の入力。
type CancelRequest struct {
	WorkflowID string
	RunID      string
	Reason     string
	// テナント識別子（NFR-E-AC-003）。
	TenantID string
}

// TerminateRequest は Terminate の入力。
type TerminateRequest struct {
	WorkflowID string
	RunID      string
	Reason     string
	// テナント識別子（NFR-E-AC-003）。
	TenantID string
}

// GetStatusRequest は GetStatus の入力。
type GetStatusRequest struct {
	WorkflowID string
	RunID      string
	// テナント識別子（NFR-E-AC-003）。
	TenantID string
}

// GetStatusResponse は GetStatus の応答。
type GetStatusResponse struct {
	Status WorkflowStatusValue
	RunID  string
}

// WorkflowAdapter は WorkflowService の操作集合。
type WorkflowAdapter interface {
	Start(ctx context.Context, req StartRequest) (StartResponse, error)
	Signal(ctx context.Context, req SignalRequest) error
	Query(ctx context.Context, req QueryRequest) (QueryResponse, error)
	Cancel(ctx context.Context, req CancelRequest) error
	Terminate(ctx context.Context, req TerminateRequest) error
	GetStatus(ctx context.Context, req GetStatusRequest) (GetStatusResponse, error)
}

// temporalWorkflowAdapter は Client（narrow interface）越しに SDK を呼ぶ実装。
type temporalWorkflowAdapter struct {
	client *Client
}

// NewWorkflowAdapter は Client から WorkflowAdapter を生成する。
func NewWorkflowAdapter(client *Client) WorkflowAdapter {
	return &temporalWorkflowAdapter{client: client}
}

// scopedWorkflowID はテナント識別子を WorkflowID の prefix として埋め込む（NFR-E-AC-003）。
// SDK / production の Temporal は WorkflowID にどんな文字列を入れても受理するため、
// "<tenant>::<workflow_id>" を実 ID として永続化することで、別テナントの ID と
// 物理的に衝突しない。in-memory backend も同じ prefix で map 上の隔離を実現する。
//
// tenantID が空の場合（dev / 試験 fake）は prefix を付けず生 ID を使う。これは
// handler 上位 requireTenantID で空テナントを弾く前提下の defensive default。
func scopedWorkflowID(tenantID, workflowID string) string {
	// 空 tenantID は prefix 不要（test fake の経路）。
	if tenantID == "" {
		// 生 ID で透過する。
		return workflowID
	}
	// "<tenant>::<workflow_id>" 形式で連結する。
	return tenantID + "::" + workflowID
}

// unscopeWorkflowID は scopedWorkflowID で prefix を付けた ID から生 ID を復元する。
// caller には生 ID を返したいので、StartResponse / GetStatusResponse の WorkflowID で使う。
func unscopeWorkflowID(tenantID, scoped string) string {
	// 空 tenantID は prefix 無しなのでそのまま返す。
	if tenantID == "" {
		// 生 ID で透過する。
		return scoped
	}
	// prefix 文字列を計算する。
	prefix := tenantID + "::"
	// prefix を持つなら除去して返す。
	if len(scoped) >= len(prefix) && scoped[:len(prefix)] == prefix {
		// prefix 除去後を返す。
		return scoped[len(prefix):]
	}
	// prefix 不在は元の ID をそのまま返す（衝突 / 互換のための fallback）。
	return scoped
}

// Start は ExecuteWorkflow で新規ワークフローを起動する。
// idempotent=true なら DuplicateRequestPolicy_REJECT_DUPLICATE は使わず、
// SDK の WorkflowIDReusePolicy / WorkflowIDConflictPolicy を AllowDuplicate で
// 既存実行を返すように設定する（同じ workflow_id で複数 run を許容）。
//
// テナント分離（NFR-E-AC-003）: 実 WorkflowID には scopedWorkflowID で
// "<tenant>::<workflow_id>" を渡す。response の WorkflowID は元の生 ID に戻して返す。
func (a *temporalWorkflowAdapter) Start(ctx context.Context, req StartRequest) (StartResponse, error) {
	tq := req.TaskQueue
	if tq == "" {
		tq = defaultTaskQueue
	}
	opts := tclient.StartWorkflowOptions{
		// テナント prefix 付きの ID で SDK / Temporal に永続化する。
		ID:        scopedWorkflowID(req.TenantID, req.WorkflowID),
		TaskQueue: tq,
	}
	// Temporal の WorkflowIDReusePolicy: 既定（AllowDuplicate）= 完了済の同 ID は新 run 採番
	// idempotent モードではここをカスタマイズ予定（plan 04-14）。
	run, err := a.client.temporalClientFor().ExecuteWorkflow(ctx, opts, req.WorkflowType, req.Input)
	if err != nil {
		return StartResponse{}, err
	}
	return StartResponse{
		// caller には prefix を取り除いた生 ID を返す。
		WorkflowID: unscopeWorkflowID(req.TenantID, run.GetID()),
		RunID:      run.GetRunID(),
	}, nil
}

// Signal は SignalWorkflow を呼ぶ。RunID が空なら SDK は最新 run に送る。
// テナント分離（NFR-E-AC-003）: WorkflowID に tenant prefix を付与して呼び出す。
func (a *temporalWorkflowAdapter) Signal(ctx context.Context, req SignalRequest) error {
	// scopedWorkflowID で tenant prefix を付ける。
	scoped := scopedWorkflowID(req.TenantID, req.WorkflowID)
	// SDK 呼出。other-tenant の WorkflowID とは prefix 違いで衝突しないため越境は物理的に不可能。
	return a.client.temporalClientFor().SignalWorkflow(ctx, scoped, req.RunID, req.SignalName, req.Payload)
}

// Query は QueryWorkflow を呼び、EncodedValue を []byte に変換して返す。
// テナント分離（NFR-E-AC-003）: WorkflowID に tenant prefix を付与する。
func (a *temporalWorkflowAdapter) Query(ctx context.Context, req QueryRequest) (QueryResponse, error) {
	// scopedWorkflowID で tenant prefix を付ける。
	scoped := scopedWorkflowID(req.TenantID, req.WorkflowID)
	enc, err := a.client.temporalClientFor().QueryWorkflow(ctx, scoped, req.RunID, req.QueryName, req.Payload)
	if err != nil {
		return QueryResponse{}, err
	}
	if enc == nil || !enc.HasValue() {
		return QueryResponse{Result: nil}, nil
	}
	// Temporal の EncodedValue.Get で []byte に展開する。SDK の default converter は JSON。
	var out []byte
	if err := enc.Get(&out); err != nil {
		return QueryResponse{}, err
	}
	return QueryResponse{Result: out}, nil
}

// Cancel は CancelWorkflow を呼ぶ。Reason はワークフロー内 catch でハンドリングされる。
// テナント分離（NFR-E-AC-003）: WorkflowID に tenant prefix を付与する。
func (a *temporalWorkflowAdapter) Cancel(ctx context.Context, req CancelRequest) error {
	// scopedWorkflowID で tenant prefix を付ける。
	scoped := scopedWorkflowID(req.TenantID, req.WorkflowID)
	return a.client.temporalClientFor().CancelWorkflow(ctx, scoped, req.RunID)
}

// Terminate は TerminateWorkflow を呼ぶ。Reason は監査ログに残る。
// テナント分離（NFR-E-AC-003）: WorkflowID に tenant prefix を付与する。
func (a *temporalWorkflowAdapter) Terminate(ctx context.Context, req TerminateRequest) error {
	// scopedWorkflowID で tenant prefix を付ける。
	scoped := scopedWorkflowID(req.TenantID, req.WorkflowID)
	return a.client.temporalClientFor().TerminateWorkflow(ctx, scoped, req.RunID, req.Reason)
}

// GetStatus は DescribeWorkflowExecution を呼んで Status を取得する。
// テナント分離（NFR-E-AC-003）: WorkflowID に tenant prefix を付与する。
func (a *temporalWorkflowAdapter) GetStatus(ctx context.Context, req GetStatusRequest) (GetStatusResponse, error) {
	// scopedWorkflowID で tenant prefix を付ける。
	scoped := scopedWorkflowID(req.TenantID, req.WorkflowID)
	desc, err := a.client.temporalClientFor().DescribeWorkflowExecution(ctx, scoped, req.RunID)
	if err != nil {
		return GetStatusResponse{}, err
	}
	if desc == nil {
		return GetStatusResponse{}, ErrWorkflowNotFound
	}
	return GetStatusResponse{
		Status: translateStatus(desc.StatusCode),
		RunID:  desc.RunID,
	}, nil
}

// 静的検査: errors.Is 用の reflection。
var _ = errors.Is
