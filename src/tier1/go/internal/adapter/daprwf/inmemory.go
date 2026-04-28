// 本ファイルは Dapr Workflow を持たない開発 / CI 環境向けの in-memory backend。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/06_Workflow_API.md
//     - FR-T1-WORKFLOW-001（短期 Workflow は Dapr Workflow building block）
//
// 役割:
//   Dapr sidecar が起動していない環境（dev / CI）でも cmd/workflow バイナリが
//   `BACKEND_DAPR` 指定で実値を返せるよう、`WorkflowAdapter` の最小実装を提供する。
//   production では `DAPR_GRPC_ENDPOINT` を結線した adapter に差し替える（plan 04-14）。
//
// 制約:
//   - workflow code を実行しない（履歴に記録するのみ）
//   - 永続化なし（再起動で全 run 消失）
//   - signal / query は履歴追記のみ、cron / continue-as-new などは未対応

package daprwf

import (
	// 全 RPC で context を伝搬する。
	"context"
	// run ID 生成にランダム値を使う。
	"crypto/rand"
	// run ID を 16 進文字列化する。
	"encoding/hex"
	// 並行制御に Mutex を使う。
	"sync"
)

// inMemoryRun は 1 件の workflow 実行（run）を表す内部レコード。
type inMemoryRun struct {
	workflowID string
	runID      string
	tenantID   string
	status     WorkflowStatusValue
}

// InMemoryWorkflow は外部 Dapr backend を持たない最小 WorkflowAdapter 実装。
type InMemoryWorkflow struct {
	mu sync.Mutex
	// workflow_id ごとの最新 run。
	runs map[string]*inMemoryRun
}

// NewInMemoryWorkflow は空 backend を生成する。
func NewInMemoryWorkflow() *InMemoryWorkflow {
	return &InMemoryWorkflow{runs: map[string]*inMemoryRun{}}
}

// nextRunID は 16 byte の crypto/rand を hex で返す。
func nextRunID() string {
	// 16 byte 乱数。
	buf := make([]byte, 16)
	// crypto/rand から読み込む（buf 満杯では nil 以外あり得ない）。
	_, _ = rand.Read(buf)
	// hex で返す。
	return hex.EncodeToString(buf)
}

// Start は新規 run を作成する。idempotent=true で同 workflow_id があれば既存を返す。
func (m *InMemoryWorkflow) Start(_ context.Context, req StartRequest) (StartResponse, error) {
	// 排他で run を作る / 取り出す。
	m.mu.Lock()
	defer m.mu.Unlock()
	// idempotent && 既存があれば既存を返す。
	if req.Idempotent && req.WorkflowID != "" {
		if existing, ok := m.runs[req.WorkflowID]; ok {
			// 既存 workflowID/runID をそのまま返す。
			return StartResponse{WorkflowID: existing.workflowID, RunID: existing.runID}, nil
		}
	}
	// workflow_id 未指定なら採番する。
	wid := req.WorkflowID
	if wid == "" {
		wid = "wf-" + nextRunID()
	}
	// 新 run を生成する。
	r := &inMemoryRun{
		workflowID: wid,
		runID:      nextRunID(),
		tenantID:   req.TenantID,
		status:     StatusRunning,
	}
	m.runs[wid] = r
	// 結果を返す。
	return StartResponse{WorkflowID: wid, RunID: r.runID}, nil
}

// resolveLocked はテナント境界（NFR-E-AC-003）を検査しつつ run を取り出す。
// run 不在 / tenant_id が run.tenantID と異なる場合は ErrNotFound を返す（other-tenant の
// 存在を漏らさないため、PermissionDenied ではなく NotFound を採用）。
// 呼出側で m.mu を握っている前提（_Locked サフィックスで明示）。
func (m *InMemoryWorkflow) resolveLocked(workflowID, tenantID string) (*inMemoryRun, bool) {
	// workflow_id で lookup する。
	r, ok := m.runs[workflowID]
	// 不在は false（呼出側 ErrNotFound 翻訳）。
	if !ok {
		// nil で false を返す。
		return nil, false
	}
	// tenant_id 不一致も false（other-tenant の workflow を漏らさない）。
	// HealthService probe など tenantID="" の経路は probe workflow が存在しない前提のため、
	// 通常運用では tenantID 必須（handler 上位 requireTenantID で強制済）。
	if r.tenantID != tenantID {
		// nil で false を返す。
		return nil, false
	}
	// 一致時は run を返す。
	return r, true
}

// Signal は履歴追記のみ（in-memory なので副作用なし）。未存在 / 越境は ErrNotFound。
func (m *InMemoryWorkflow) Signal(_ context.Context, req SignalRequest) error {
	m.mu.Lock()
	defer m.mu.Unlock()
	// テナント境界検査つき lookup。
	if _, ok := m.resolveLocked(req.WorkflowID, req.TenantID); !ok {
		// 不在 or 越境は NotFound 翻訳。
		return ErrNotFound
	}
	// signal 受信は履歴に追記したいが、in-memory backend は履歴を保持しない。no-op。
	return nil
}

// Query は履歴追記のみ。Result は空 bytes（dev / CI で query handler は未起動）。
func (m *InMemoryWorkflow) Query(_ context.Context, req QueryRequest) (QueryResponse, error) {
	m.mu.Lock()
	defer m.mu.Unlock()
	// テナント境界検査つき lookup。
	if _, ok := m.resolveLocked(req.WorkflowID, req.TenantID); !ok {
		// 不在 or 越境は NotFound 翻訳。
		return QueryResponse{}, ErrNotFound
	}
	// 空 bytes（query handler が無いため）。
	return QueryResponse{Result: nil}, nil
}

// Cancel は status を Canceled に遷移させる。越境 / 不在は ErrNotFound。
func (m *InMemoryWorkflow) Cancel(_ context.Context, req CancelRequest) error {
	m.mu.Lock()
	defer m.mu.Unlock()
	// テナント境界検査つき lookup。
	r, ok := m.resolveLocked(req.WorkflowID, req.TenantID)
	if !ok {
		// 不在 or 越境は NotFound 翻訳。
		return ErrNotFound
	}
	r.status = StatusCanceled
	return nil
}

// Terminate は status を Terminated に遷移させる。越境 / 不在は ErrNotFound。
func (m *InMemoryWorkflow) Terminate(_ context.Context, req TerminateRequest) error {
	m.mu.Lock()
	defer m.mu.Unlock()
	// テナント境界検査つき lookup。
	r, ok := m.resolveLocked(req.WorkflowID, req.TenantID)
	if !ok {
		// 不在 or 越境は NotFound 翻訳。
		return ErrNotFound
	}
	r.status = StatusTerminated
	return nil
}

// GetStatus は現在 status と run_id を返す。越境 / 不在は ErrNotFound。
func (m *InMemoryWorkflow) GetStatus(_ context.Context, req GetStatusRequest) (GetStatusResponse, error) {
	m.mu.Lock()
	defer m.mu.Unlock()
	// テナント境界検査つき lookup。
	r, ok := m.resolveLocked(req.WorkflowID, req.TenantID)
	if !ok {
		// 不在 or 越境は NotFound 翻訳。
		return GetStatusResponse{}, ErrNotFound
	}
	return GetStatusResponse{Status: r.status, RunID: r.runID}, nil
}
