// lib.rs — tier2 Temporal Workflow ルートモジュール（設計方針 13 / 18 / 25）
// Saga / 夜間 close / 月次集計 / 年次 archive / L1+ 年次 dry_run の 5 Workflow を公開する

// Saga 長時間 Workflow モジュール
pub mod saga_long_running;
// 夜間 close バッチ Workflow モジュール
pub mod nightly_close;
// 月次集計バッチ Workflow モジュール
pub mod monthly_aggregation;
// 年次 archive Workflow モジュール
pub mod yearly_archive;
// L1+ pair 年次 dry_run Workflow モジュール
pub mod yearly_dry_run;

// WorkflowError は全 Workflow 共通のエラー型を宣言する
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    // TimeoutError: Workflow がタイムアウトした場合のエラー
    #[error("Workflow timeout: {workflow_id} exceeded {timeout_secs}s")]
    Timeout {
        // workflow_id: タイムアウトした Workflow ID
        workflow_id: String,
        // timeout_secs: タイムアウト秒数
        timeout_secs: u64,
    },
    // CompensationError: Saga compensation に失敗した場合のエラー
    #[error("Saga compensation failed: {step} — {cause}")]
    CompensationFailed {
        // step: 失敗した compensation ステップ名
        step: String,
        // cause: 原因エラーメッセージ
        cause: String,
    },
    // DatabaseError: DB 操作エラー
    #[error("Database error in workflow: {0}")]
    Database(String),
    // ExternalServiceError: 外部サービス呼び出しエラー
    #[error("External service error: {service} — {message}")]
    ExternalService {
        // service: エラーが発生した外部サービス名
        service: String,
        // message: エラーメッセージ
        message: String,
    },
}
