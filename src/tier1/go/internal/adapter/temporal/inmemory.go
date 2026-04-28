// 本ファイルは Temporal 接続を行わない in-memory backend。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/06_Workflow_API.md
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md
//     - DS-SW-COMP-010（t1-workflow Pod、Dapr Workflow / Temporal pluggable）
//
// 役割:
//   Temporal Server を持たない開発 / CI 環境でも cmd/workflow バイナリが起動から
//   gRPC 応答まで実値を返せるよう、`temporalClient` interface を満たす in-memory 実装を
//   提供する。production では `TEMPORAL_HOSTPORT` 環境変数で実 Temporal に切替わる。
//
// 制限事項（in-memory backend は POC / dev 用途）:
//   - workflow 関数の実行は行わない（ProcessOrder 等の引数は記録するのみ）
//   - 永続化なし（再起動で全 workflow 状態消失）
//   - 並行制御は単純 Mutex（production の Temporal は MVCC + history sharding）
//   - signal / query は履歴に追記するが、actor model / cron 等の高度機能は未対応
//
// happy-path セマンティクス:
//   - Start: 新規 workflow run を生成、status=Running、入力を保持
//   - Signal: signal 名と payload を履歴に追加
//   - Query: query 名 + payload を履歴に追加し、空 result を返す
//   - Cancel: status=Canceled に遷移
//   - Terminate: status=Terminated に遷移
//   - GetStatus: 現状の status と RunID を返す

package temporal

import (
	// 全 RPC で context を伝搬する。
	"context"
	// run ID 生成にランダム値を使う（crypto/rand で衝突確率を抑える）。
	"crypto/rand"
	// run ID を 16 進文字列化する。
	"encoding/hex"
	// 並行制御に Mutex を使う。
	"sync"

	// Temporal SDK の StartWorkflowOptions / WorkflowRun 型を参照する。
	tclient "go.temporal.io/sdk/client"
	// Query 応答の EncodedValue 型を参照する。
	"go.temporal.io/sdk/converter"
)

// inMemoryRun は 1 つの workflow 実行（run）を表す。
type inMemoryRun struct {
	// workflow ID（呼出側が指定 / 未指定なら同 ID で連結 run になる）。
	workflowID string
	// run ID（バイナリランダム 16 byte の hex 文字列）。
	runID string
	// workflow 種別名（"ProcessOrder" 等）。
	workflowType string
	// 起動入力（adapter 層で []byte として渡される）。
	input interface{}
	// signal 履歴（signal 名 → 最後の payload。複数 payload は最新で上書き）。
	signals map[string]interface{}
	// 現在 status（Temporal SDK の WorkflowExecutionStatus 整数値）。
	status int32
	// terminate / cancel 時の理由（GetStatus は exposing しないが debug 用）。
	reason string
}

// InMemoryTemporal は Temporal Server なしで動く temporalClient 実装。
type InMemoryTemporal struct {
	// mu は全操作を直列化する Mutex。
	mu sync.Mutex
	// runs は workflowID → 最新 run のマップ。
	runs map[string]*inMemoryRun
}

// NewInMemoryTemporal は空の InMemoryTemporal を生成する。
func NewInMemoryTemporal() *InMemoryTemporal {
	// 空 map で初期化する。
	return &InMemoryTemporal{runs: map[string]*inMemoryRun{}}
}

// generateRunID は 16 byte ランダム値を hex 文字列で返す。
// 衝突確率 2^-128 を許容する（production の Temporal は UUIDv4 相当）。
func generateRunID() string {
	// 16 byte の buffer を確保する。
	buf := make([]byte, 16)
	// crypto/rand から読み込む（失敗時は固定値で fallback）。
	if _, err := rand.Read(buf); err != nil {
		// fallback として固定 sentinel を返す（panic ではなく観測可能エラーに留める）。
		return "00000000000000000000000000000000"
	}
	// 32 文字の hex 文字列に変換する。
	return hex.EncodeToString(buf)
}

// inMemoryWorkflowRun は tclient.WorkflowRun interface の最小 in-memory 実装。
// Get / GetWithOptions は workflow 完了を待つ意味論だが、in-memory backend では
// 即時 nil を返す（workflow 関数が実行されないため結果はない）。
type inMemoryWorkflowRun struct {
	// workflow ID。
	id string
	// run ID。
	runID string
}

// GetID は workflow ID を返す。
func (r *inMemoryWorkflowRun) GetID() string { return r.id }

// GetRunID は run ID を返す。
func (r *inMemoryWorkflowRun) GetRunID() string { return r.runID }

// Get は workflow 完了結果を valuePtr に詰める interface contract。
// in-memory backend は実 workflow 実行を持たないため、即時 nil（成功扱い）を返す。
func (r *inMemoryWorkflowRun) Get(_ context.Context, _ interface{}) error { return nil }

// GetWithOptions も同様に即時 nil。
func (r *inMemoryWorkflowRun) GetWithOptions(_ context.Context, _ interface{}, _ tclient.WorkflowRunGetOptions) error {
	return nil
}

// inMemoryEncodedValue は converter.EncodedValue interface の最小 fake。
// in-memory backend の Query は履歴記録のみで実 workflow に値を持たないため、
// 常に HasValue=false を返し、Get は no-op で nil を返す。
type inMemoryEncodedValue struct{}

// HasValue は常に false を返す（in-memory には値がない）。
func (inMemoryEncodedValue) HasValue() bool { return false }

// Get は valuePtr を変更せず nil を返す。
func (inMemoryEncodedValue) Get(_ interface{}) error { return nil }

// ExecuteWorkflow は新規 workflow run を生成して runs に登録する。
func (m *InMemoryTemporal) ExecuteWorkflow(_ context.Context, options tclient.StartWorkflowOptions, workflow interface{}, args ...interface{}) (tclient.WorkflowRun, error) {
	// Mutex で runs を保護する。
	m.mu.Lock()
	defer m.mu.Unlock()
	// run ID を新規発行する。
	runID := generateRunID()
	// workflow 種別を string として取り出す（adapter 経由では string が渡される慣行）。
	wfType := ""
	// type assertion で string を取り出す。
	if s, ok := workflow.(string); ok {
		// 受領した workflow 種別を保持する。
		wfType = s
	}
	// 入力 args 0 番目を入力本文として保持する。
	var input interface{}
	// args が 1 件以上なら最初を採用する。
	if len(args) > 0 {
		// 入力本文を保持する。
		input = args[0]
	}
	// inMemoryRun を生成して登録する。
	run := &inMemoryRun{
		// 呼出側指定の workflow ID（空文字なら無名 run、production と同じ仕様）。
		workflowID: options.ID,
		// 発行済 run ID。
		runID: runID,
		// 種別。
		workflowType: wfType,
		// 入力本文。
		input: input,
		// signal 履歴 map を初期化する。
		signals: map[string]interface{}{},
		// status を Running に初期化する。
		status: temporalStatusRunning,
	}
	// runs に最新 run として登録する。
	m.runs[options.ID] = run
	// SDK に返す WorkflowRun を生成する。
	return &inMemoryWorkflowRun{id: run.workflowID, runID: run.runID}, nil
}

// SignalWorkflow は signal を履歴に追加する。
// 該当 workflow が未存在なら Temporal SDK 互換の error を返す（in-memory では sentinel）。
func (m *InMemoryTemporal) SignalWorkflow(_ context.Context, workflowID, _ string, signalName string, arg interface{}) error {
	// Mutex で runs を保護する。
	m.mu.Lock()
	defer m.mu.Unlock()
	// 該当 workflow を取り出す。
	run, ok := m.runs[workflowID]
	// 未存在は ErrWorkflowNotFound を返す。
	if !ok {
		// sentinel error を返す。
		return ErrWorkflowNotFound
	}
	// signal を履歴 map に上書き登録する。
	run.signals[signalName] = arg
	// 成功時 nil を返す。
	return nil
}

// QueryWorkflow は query を実行する。in-memory は値を持たないため空の EncodedValue を返す。
func (m *InMemoryTemporal) QueryWorkflow(_ context.Context, workflowID, _, _ string, _ ...interface{}) (converter.EncodedValue, error) {
	// Mutex で runs を保護する。
	m.mu.Lock()
	defer m.mu.Unlock()
	// 該当 workflow を取り出す。
	_, ok := m.runs[workflowID]
	// 未存在は ErrWorkflowNotFound を返す。
	if !ok {
		// sentinel error を返す。
		return nil, ErrWorkflowNotFound
	}
	// in-memory は値を持たないため HasValue=false の EncodedValue を返す。
	return inMemoryEncodedValue{}, nil
}

// CancelWorkflow は status を Canceled に遷移させる。
func (m *InMemoryTemporal) CancelWorkflow(_ context.Context, workflowID, _ string) error {
	// Mutex で runs を保護する。
	m.mu.Lock()
	defer m.mu.Unlock()
	// 該当 workflow を取り出す。
	run, ok := m.runs[workflowID]
	// 未存在は ErrWorkflowNotFound を返す。
	if !ok {
		// sentinel error を返す。
		return ErrWorkflowNotFound
	}
	// status を Canceled に変更する。
	run.status = temporalStatusCanceled
	// 成功時 nil を返す。
	return nil
}

// TerminateWorkflow は status を Terminated に遷移させる。
func (m *InMemoryTemporal) TerminateWorkflow(_ context.Context, workflowID, _, reason string, _ ...interface{}) error {
	// Mutex で runs を保護する。
	m.mu.Lock()
	defer m.mu.Unlock()
	// 該当 workflow を取り出す。
	run, ok := m.runs[workflowID]
	// 未存在は ErrWorkflowNotFound を返す。
	if !ok {
		// sentinel error を返す。
		return ErrWorkflowNotFound
	}
	// status を Terminated に変更する。
	run.status = temporalStatusTerminated
	// reason を記録する。
	run.reason = reason
	// 成功時 nil を返す。
	return nil
}

// DescribeWorkflowExecution は status と RunID を返す。
func (m *InMemoryTemporal) DescribeWorkflowExecution(_ context.Context, workflowID, _ string) (*describeResponse, error) {
	// Mutex で runs を保護する。
	m.mu.Lock()
	defer m.mu.Unlock()
	// 該当 workflow を取り出す。
	run, ok := m.runs[workflowID]
	// 未存在は ErrWorkflowNotFound を返す。
	if !ok {
		// sentinel error を返す。
		return nil, ErrWorkflowNotFound
	}
	// describeResponse を返す。
	return &describeResponse{
		// status code を Temporal の整数値で返す。
		StatusCode: run.status,
		// 最新 run ID。
		RunID: run.runID,
	}, nil
}

// NewClientWithInMemory は in-memory backend を持つ Client を生成する。
// cmd/workflow/main.go から TEMPORAL_HOSTPORT 未設定時の fallback として呼ばれる。
func NewClientWithInMemory() *Client {
	// 空 in-memory backend を生成する。
	mem := NewInMemoryTemporal()
	// Client に in-memory backend を埋め込む（hostPort は識別ラベル）。
	return &Client{
		// 観測用ラベル。
		hostPort: "in-memory",
		// in-memory implementation を temporalClient として割当てる。
		tc: mem,
		// closer は不要（in-memory は GC 任せ）。
	}
}
