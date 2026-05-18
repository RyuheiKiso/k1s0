// workflow.rs — k1s0 tier1 Library backend: Workflow/Long-running Saga L1+ trait
// backend 専用カテゴリ（frontend には提供しない）。
// L1+: OSS の全機能を表現 + tier1 横断要素（auth context 伝播 / retry / tracing）を強制。
// 公開 API に OSS 型（temporal.io sdk / Conductor 等）を露出しない。
// Saga パターン（分散 transaction の補償）と Long-running workflow を統一的に扱う。

// async_trait: async fn in trait を stable で使用するためのマクロ
use async_trait::async_trait;
// anyhow: エラーハンドリング（Result 型の統一）
use anyhow::Result;
// serde: ワークフロー定義 / 状態のシリアライズに使用する
use serde::{Deserialize, Serialize};
// AuthContext: ワークフロー操作に認証コンテキストを伝播する
use crate::core::auth::AuthContext;
// SpanContext: ワークフロー操作に tracing コンテキストを伝播する
use crate::core::observability::SpanContext;

// WorkflowStatus はワークフローの実行ステータスを表す Library 独自型。
// Temporal の WorkflowExecutionStatus を Library 独自語彙で表現する。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    // Running: 実行中（step が進行中）
    Running,
    // Completed: 正常完了
    Completed,
    // Failed: 失敗（補償実行が必要な場合は Compensating に遷移する）
    Failed(String),
    // Compensating: 補償実行中（Saga の rollback に相当）
    Compensating,
    // Compensated: 補償完了（全補償ステップが正常完了した）
    Compensated,
    // TimedOut: タイムアウト（SLA 超過）
    TimedOut,
    // Cancelled: キャンセル（外部から明示的にキャンセルされた）
    Cancelled,
    // Paused: 一時停止（human approval 待ち等）
    Paused,
}

// WorkflowInstance はワークフローのインスタンス情報を表す Library 独自型。
// OSS の WorkflowExecution / WorkflowHandle を Library 独自型に変換して露出しない。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInstance {
    // workflow_id: ワークフローインスタンスの識別子（UUID v7 形式）
    pub workflow_id: String,
    // workflow_type: ワークフロー種別（例: "order_fulfillment_saga"）
    pub workflow_type: String,
    // status: 現在のステータス
    pub status: WorkflowStatus,
    // tenant_id: このワークフローが属するテナントの識別子
    pub tenant_id: String,
    // current_step: 現在実行中のステップ名（None は開始前 / 完了後）
    pub current_step: Option<String>,
    // started_at_hlc_tick: 開始時刻の HLC tick 数（wall-clock TTL 禁止）
    pub started_at_hlc_tick: u64,
    // completed_at_hlc_tick: 完了時刻の HLC tick 数（None は未完了）
    pub completed_at_hlc_tick: Option<u64>,
    // payload: ワークフローの入力 / 出力ペイロード（JSON 形式）
    pub payload: serde_json::Value,
}

// StepResult はワークフローの 1 ステップの実行結果を表す struct。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    // step_name: ステップ名
    pub step_name: String,
    // success: ステップが正常に完了したかどうか
    pub success: bool,
    // output: ステップの出力（次のステップへの入力に使用する）
    pub output: serde_json::Value,
    // error_message: 失敗時のエラーメッセージ（success=true の場合は None）
    pub error_message: Option<String>,
}

// WorkflowSignal はワークフローへの外部シグナルを表す Library 独自型。
// Human approval / external event の受信に使用する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSignal {
    // signal_type: シグナルの種別（例: "human_approved" / "payment_completed"）
    pub signal_type: String,
    // payload: シグナルのペイロード（JSON 形式）
    pub payload: serde_json::Value,
    // sender_subject_id: シグナルを送った主体の subject_id（監査用）
    pub sender_subject_id: String,
}

// WorkflowOrchestrator は Workflow/Long-running Saga の L1+ 抽象 trait。
// Temporal / Conductor / AWS Step Functions 等を実装で切り替えられる。
// auth context 伝播 / tracing は強制する。
#[async_trait]
pub trait WorkflowOrchestrator: Send + Sync {
    // start はワークフローを開始する。
    // auth_ctx は tenant_id の境界を保証し、audit ログに使用する。
    // span_ctx は tracing コンテキストの伝播に使用する。
    async fn start(
        &self,
        workflow_type: &str,
        payload: serde_json::Value,
        auth_ctx: &AuthContext,
        span_ctx: Option<&SpanContext>,
    ) -> Result<WorkflowInstance>;

    // get は workflow_id のインスタンス情報を返す。
    // auth_ctx は tenant_id の境界を保証する。
    async fn get(
        &self,
        workflow_id: &str,
        auth_ctx: &AuthContext,
    ) -> Result<Option<WorkflowInstance>>;

    // signal は実行中のワークフローに外部シグナルを送る。
    // Paused 状態からの再開（human approval 完了等）に使用する。
    async fn signal(
        &self,
        workflow_id: &str,
        signal: WorkflowSignal,
        auth_ctx: &AuthContext,
    ) -> Result<()>;

    // cancel はワークフローをキャンセルする（補償ステップを実行してから停止する）。
    async fn cancel(
        &self,
        workflow_id: &str,
        reason: &str,
        auth_ctx: &AuthContext,
    ) -> Result<WorkflowInstance>;

    // list は tenant_id のワークフローインスタンス一覧を返す（ページネーション対応）。
    async fn list(
        &self,
        status_filter: Option<WorkflowStatus>,
        page_size: u32,
        cursor: Option<String>,
        auth_ctx: &AuthContext,
    ) -> Result<(Vec<WorkflowInstance>, Option<String>)>;
}
