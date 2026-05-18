// workflow.go — k1s0 tier1 Library Go 実装: Workflow / Long-running Saga の L1+ interface
// 16_ワークフロー適合仕様.md §WorkflowClient（Temporal L1+ 深耕）に準拠する。
// Temporal の full API を Library 独自語彙で表現しつつ、AuthContext 伝播と tenant 分離を強制する。
// OSS 型（temporal-go / temporal.Client 等）を公開シグネチャに一切含まない。

// パッケージ名: keyhandle（tier1 Library の Workflow / Saga API を提供する）
package keyhandle

import (
	// context: context.Context（AuthContext 伝播 + 非同期操作に使用する）
	"context"
)

// WorkflowStatus はワークフロー実行の状態を宣言する型。
// Temporal の workflow.Execution Status に準拠した Library 独自語彙とする。
type WorkflowStatus string

const (
	// WorkflowStatusRunning: 実行中
	WorkflowStatusRunning WorkflowStatus = "running"
	// WorkflowStatusCompleted: 正常完了
	WorkflowStatusCompleted WorkflowStatus = "completed"
	// WorkflowStatusFailed: 失敗（リトライ上限超過）
	WorkflowStatusFailed WorkflowStatus = "failed"
	// WorkflowStatusCanceled: キャンセルされた
	WorkflowStatusCanceled WorkflowStatus = "canceled"
	// WorkflowStatusTimedOut: タイムアウト
	WorkflowStatusTimedOut WorkflowStatus = "timed_out"
	// WorkflowStatusContinuedAsNew: 継続（ContinueAsNew パターン）
	WorkflowStatusContinuedAsNew WorkflowStatus = "continued_as_new"
	// WorkflowStatusTerminated: 強制終了
	WorkflowStatusTerminated WorkflowStatus = "terminated"
)

// WorkflowOptions はワークフロー開始オプションを宣言する型。
// Temporal の StartWorkflowOptions を Library 独自語彙に翻訳する。
type WorkflowOptions struct {
	// WorkflowID: ワークフロー識別子（idempotency key として使用する: tenant prefix を含む形式を推奨する）
	WorkflowID string
	// TaskQueue: 実行するタスクキュー名（Worker が listen するキュー）
	TaskQueue string
	// MaxRetries: ワークフロー全体のリトライ上限回数（0 = リトライなし）
	MaxRetries int
	// RetentionDays: ワークフロー履歴の保持日数（0 = デフォルト設定を使用する）
	RetentionDays int
	// SearchAttributes: Temporal Search Attribute（ダッシュボード検索用のインデックス対象属性）
	SearchAttributes map[string]any
	// Memo: ワークフローメモ（非インデックスの補足情報）
	Memo map[string]any
}

// ActivityOptions はアクティビティ実行オプションを宣言する型。
// Temporal の ActivityOptions を Library 独自語彙に翻訳する。
type ActivityOptions struct {
	// TaskQueue: アクティビティを実行するタスクキュー名（空 = ワークフローのキューを継承する）
	TaskQueue string
	// ScheduleToCloseTimeoutMs: スケジュールから完了までの全体タイムアウト（ミリ秒）
	ScheduleToCloseTimeoutMs int64
	// StartToCloseTimeoutMs: 実行開始から完了までのタイムアウト（ミリ秒）
	StartToCloseTimeoutMs int64
	// MaxRetries: アクティビティのリトライ上限回数
	MaxRetries int
	// RetryBackoffMs: リトライのバックオフ間隔（ミリ秒）
	RetryBackoffMs int64
	// HeartbeatTimeoutMs: ハートビートタイムアウト（long-running アクティビティに使用する）
	HeartbeatTimeoutMs int64
}

// WorkflowExecution はワークフロー実行の識別子を宣言する型。
// Temporal の workflow.Execution を Library 独自語彙で表現する。
type WorkflowExecution struct {
	// WorkflowID: ワークフロー識別子
	WorkflowID string
	// RunID: 実行 ID（同一 WorkflowID のリトライ・再実行を区別する）
	RunID string
	// TenantID: ワークフローの所属テナント識別子
	TenantID string
}

// WorkflowRun はワークフロー実行ハンドルを宣言する interface。
// Temporal の client.WorkflowRun を Library 独自語彙で表現する。
type WorkflowRun interface {
	// GetID はワークフロー識別子を返す。
	GetID() string

	// GetRunID は実行 ID を返す。
	GetRunID() string

	// Get はワークフローの完了を待機して結果を取得する。
	// valuePtr は結果を格納するポインター（nil = 結果を取得しない）。
	Get(ctx context.Context, valuePtr any) error

	// Status はワークフローの現在の状態を返す。
	Status(ctx context.Context) (WorkflowStatus, error)
}

// WorkflowClient は Workflow / Long-running Saga の L1+ 抽象 interface を宣言する。
// Temporal の full API を Library 独自語彙で表現する。
// OSS 型（temporal.Client 等）を引数・戻り値に一切含まない。
// ctx に AuthContext が含まれることを強制する（tenant 分離必須）。
type WorkflowClient interface {
	// StartWorkflow はワークフローを開始して WorkflowRun を返す。
	// ctx には AuthContext が伝播されている前提とする（tenant 分離必須）。
	// workflowType はワークフロー登録名（Go 関数名 / 文字列 type alias）。
	// args はワークフロー開始引数（protobuf メッセージの JSON シリアライズ値を推奨する）。
	StartWorkflow(ctx context.Context, opts WorkflowOptions, workflowType string, args ...any) (WorkflowRun, error)

	// SignalWorkflow は実行中のワークフローにシグナルを送信する。
	// workflowID / runID でワークフローを特定する（runID = "" で最新の実行に送信する）。
	// signalName はシグナル名、arg はシグナル引数。
	SignalWorkflow(ctx context.Context, tenantID string, workflowID string, runID string, signalName string, arg any) error

	// QueryWorkflow は実行中のワークフローにクエリを送信して結果を取得する。
	// queryType はクエリ名、args はクエリ引数、valuePtr は結果格納ポインター。
	QueryWorkflow(ctx context.Context, tenantID string, workflowID string, runID string, queryType string, args ...any) (any, error)

	// CancelWorkflow は実行中のワークフローをキャンセルする（グレースフルキャンセル）。
	CancelWorkflow(ctx context.Context, tenantID string, workflowID string, runID string) error

	// TerminateWorkflow は実行中のワークフローを強制終了する（理由を指定する）。
	TerminateWorkflow(ctx context.Context, tenantID string, workflowID string, runID string, reason string) error

	// DescribeWorkflow はワークフローの詳細情報を取得する。
	DescribeWorkflow(ctx context.Context, tenantID string, workflowID string, runID string) (*WorkflowDescription, error)

	// Close はクライアントのリソースを解放する。
	Close()
}

// WorkflowDescription はワークフロー実行の詳細情報を宣言する型。
// Temporal の DescribeWorkflowResponse を Library 独自語彙で表現する。
type WorkflowDescription struct {
	// Execution: ワークフロー実行の識別子
	Execution WorkflowExecution
	// Status: 現在の状態
	Status WorkflowStatus
	// WorkflowType: ワークフロータイプ名
	WorkflowType string
	// StartTimeMs: 開始時刻（Unix ミリ秒）
	StartTimeMs int64
	// CloseTimeMs: 終了時刻（Unix ミリ秒: 実行中の場合は 0）
	CloseTimeMs int64
	// SearchAttributes: 検索属性
	SearchAttributes map[string]any
	// Memo: ワークフローメモ
	Memo map[string]any
}

// WorkflowWorker は Worker プロセスのワークフロー / アクティビティ実行インターフェースを宣言する。
// Temporal の worker.Worker を Library 独自語彙で抽象化する。
type WorkflowWorker interface {
	// RegisterWorkflow はワークフロー関数をワーカーに登録する。
	// workflowFn は Temporal ワークフロー関数（第一引数が workflow.Context）。
	RegisterWorkflow(workflowFn any)

	// RegisterActivity はアクティビティ関数をワーカーに登録する。
	// activityFn は Temporal アクティビティ関数（第一引数が context.Context）。
	RegisterActivity(activityFn any)

	// Start はワーカーを起動する（バックグラウンドで Temporal タスクキューを poll する）。
	Start() error

	// Stop はワーカーをグレースフルにシャットダウンする。
	Stop()
}
