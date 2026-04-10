// DTO (Data Transfer Object) モジュール
// ハンドラ間で共有されるリクエスト/レスポンス構造体とアプリケーション状態を定義する

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use k1s0_server_common::ErrorResponse;

use crate::adapter::middleware::auth::AuthState;
use crate::usecase::check_overdue_tasks::CheckOverdueTasksUseCase;
use crate::usecase::{
    ApproveTaskUseCase, CancelInstanceUseCase, CreateWorkflowUseCase, DeleteWorkflowUseCase,
    GetInstanceUseCase, GetWorkflowUseCase, ListInstancesUseCase, ListTasksUseCase,
    ListWorkflowsUseCase, ReassignTaskUseCase, RejectTaskUseCase, StartInstanceUseCase,
    UpdateWorkflowUseCase,
};

// --- アプリケーション状態 ---

/// 全ユースケースとメトリクスを保持するアプリケーション共有状態
/// `db_pool` は /readyz エンドポイントで DB 接続確認に使用する（CRITICAL-003 対応）
#[derive(Clone)]
pub struct AppState {
    pub create_workflow_uc: Arc<CreateWorkflowUseCase>,
    pub update_workflow_uc: Arc<UpdateWorkflowUseCase>,
    pub delete_workflow_uc: Arc<DeleteWorkflowUseCase>,
    pub get_workflow_uc: Arc<GetWorkflowUseCase>,
    pub list_workflows_uc: Arc<ListWorkflowsUseCase>,
    pub start_instance_uc: Arc<StartInstanceUseCase>,
    pub get_instance_uc: Arc<GetInstanceUseCase>,
    pub list_instances_uc: Arc<ListInstancesUseCase>,
    pub cancel_instance_uc: Arc<CancelInstanceUseCase>,
    pub list_tasks_uc: Arc<ListTasksUseCase>,
    pub approve_task_uc: Arc<ApproveTaskUseCase>,
    pub reject_task_uc: Arc<RejectTaskUseCase>,
    pub reassign_task_uc: Arc<ReassignTaskUseCase>,
    pub check_overdue_tasks_uc: Arc<CheckOverdueTasksUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub auth_state: Option<AuthState>,
    /// DB 接続確認用のコネクションプール（未設定の場合は None）
    pub db_pool: Option<sqlx::PgPool>,
}

impl AppState {
    /// 認証状態を設定して自身を返すビルダーメソッド
    #[must_use]
    pub fn with_auth(mut self, auth_state: AuthState) -> Self {
        self.auth_state = Some(auth_state);
        self
    }
}

// --- ワークフロー関連 DTO ---

/// ワークフロー作成リクエスト
#[derive(Debug, Deserialize)]
pub struct CreateWorkflowRequest {
    pub name: String,
    pub description: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub steps: Vec<StepRequest>,
}

/// trueを返すデフォルト値ヘルパー
pub(crate) fn default_true() -> bool {
    true
}

/// ステップ定義リクエスト（作成・更新で共用）
#[derive(Debug, Deserialize)]
pub struct StepRequest {
    pub step_id: String,
    pub name: String,
    pub step_type: String,
    pub assignee_role: Option<String>,
    pub timeout_hours: Option<u32>,
    pub on_approve: Option<String>,
    pub on_reject: Option<String>,
}

/// ワークフロー定義レスポンス
#[derive(Debug, Serialize)]
pub struct WorkflowResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: u32,
    pub enabled: bool,
    pub step_count: usize,
    pub steps: Vec<StepResponse>,
    pub created_at: String,
    pub updated_at: String,
}

/// ステップ定義レスポンス
#[derive(Debug, Serialize)]
pub struct StepResponse {
    pub step_id: String,
    pub name: String,
    pub step_type: String,
    pub assignee_role: Option<String>,
    pub timeout_hours: Option<u32>,
    pub on_approve: Option<String>,
    pub on_reject: Option<String>,
}

/// ドメインエンティティからレスポンスDTOへの変換ヘルパー
pub(crate) fn to_step_response(
    step: &crate::domain::entity::workflow_step::WorkflowStep,
) -> StepResponse {
    StepResponse {
        step_id: step.step_id.clone(),
        name: step.name.clone(),
        step_type: step.step_type.clone(),
        assignee_role: step.assignee_role.clone(),
        timeout_hours: step.timeout_hours,
        on_approve: step.on_approve.clone(),
        on_reject: step.on_reject.clone(),
    }
}

/// ワークフロー一覧取得クエリパラメータ
#[derive(Debug, Deserialize)]
pub struct ListWorkflowsQuery {
    #[serde(default = "default_false")]
    pub enabled_only: bool,
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

/// falseを返すデフォルト値ヘルパー
pub(crate) fn default_false() -> bool {
    false
}

/// ページ番号のデフォルト値（1）
pub(crate) fn default_page() -> u32 {
    1
}

/// ページサイズのデフォルト値（20）
pub(crate) fn default_page_size() -> u32 {
    20
}

/// ワークフロー一覧レスポンス
#[derive(Debug, Serialize)]
pub struct ListWorkflowsResponse {
    pub workflows: Vec<WorkflowResponse>,
    pub pagination: PaginationResponse,
}

/// ページネーション情報レスポンス
#[derive(Debug, Serialize)]
pub struct PaginationResponse {
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
}

/// ワークフロー更新リクエスト
#[derive(Debug, Deserialize)]
pub struct UpdateWorkflowRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
    #[serde(default)]
    pub steps: Option<Vec<StepRequest>>,
}

// --- インスタンス関連 DTO ---

/// ワークフロー実行（インスタンス開始）リクエスト
/// RUST-LOW-001 対応: `initiator_id` は JWT Claims の sub から取得するため省略可能とする
/// Claims が存在する場合は sub を優先し、存在しない場合はリクエストの値にフォールバックする
#[derive(Debug, Deserialize)]
pub struct ExecuteWorkflowRequest {
    pub title: String,
    /// 省略時は JWT Claims の sub から自動設定される
    #[serde(default)]
    pub initiator_id: Option<String>,
    #[serde(default)]
    pub context: serde_json::Value,
}

/// ワークフロー実行レスポンス
#[derive(Debug, Serialize)]
pub struct ExecuteWorkflowResponse {
    pub id: String,
    pub workflow_id: String,
    pub workflow_name: String,
    pub title: String,
    pub initiator_id: String,
    pub context: serde_json::Value,
    pub status: String,
    pub current_step_id: Option<String>,
    pub started_at: String,
    pub created_at: String,
    pub completed_at: Option<String>,
}

/// インスタンス状態レスポンス
#[derive(Debug, Serialize)]
pub struct InstanceStatusResponse {
    pub id: String,
    pub workflow_id: String,
    pub workflow_name: String,
    pub title: String,
    pub initiator_id: String,
    pub context: serde_json::Value,
    pub status: String,
    pub current_step_id: Option<String>,
    pub started_at: String,
    pub created_at: String,
    pub completed_at: Option<String>,
}

/// インスタンス一覧取得クエリパラメータ
#[derive(Debug, Deserialize)]
pub struct ListInstancesQuery {
    pub status: Option<String>,
    pub workflow_id: Option<String>,
    pub initiator_id: Option<String>,
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

/// インスタンスキャンセルリクエスト
#[derive(Debug, Deserialize)]
pub struct CancelInstanceRequest {
    pub reason: Option<String>,
}

// --- タスク関連 DTO ---

/// タスク一覧取得クエリパラメータ
#[derive(Debug, Deserialize)]
pub struct ListTasksQuery {
    pub assignee_id: Option<String>,
    pub status: Option<String>,
    pub instance_id: Option<String>,
    #[serde(default = "default_false")]
    pub overdue_only: bool,
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

/// タスク承認リクエスト
#[derive(Debug, Deserialize)]
pub struct ApproveTaskRequest {
    #[serde(alias = "actor_id")]
    pub actor_user_id: String,
    pub comment: Option<String>,
}

/// タスク却下リクエスト
#[derive(Debug, Deserialize)]
pub struct RejectTaskRequest {
    #[serde(alias = "actor_id")]
    pub actor_user_id: String,
    pub comment: Option<String>,
}

/// タスク再割り当てリクエスト
#[derive(Debug, Deserialize)]
pub struct ReassignTaskRequest {
    pub new_assignee_id: String,
    pub reason: Option<String>,
    #[serde(alias = "actor_id")]
    pub actor_user_id: String,
}

// --- 共通ヘルパー ---

/// エラーレスポンスを生成するヘルパー。
/// .`expect()` を排除し、ErrorResponse を直接返す。
pub(crate) fn error_json(code: &str, message: &str) -> ErrorResponse {
    ErrorResponse::new(code, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// default_true がtrueを返す
    #[test]
    fn default_true_returns_true() {
        assert!(default_true());
    }

    /// default_false がfalseを返す
    #[test]
    fn default_false_returns_false() {
        assert!(!default_false());
    }

    /// default_page が1を返す
    #[test]
    fn default_page_returns_1() {
        assert_eq!(default_page(), 1);
    }

    /// default_page_size が20を返す
    #[test]
    fn default_page_size_returns_20() {
        assert_eq!(default_page_size(), 20);
    }

    /// to_step_response が WorkflowStep から StepResponse を正しく変換する
    #[test]
    fn to_step_response_converts_correctly() {
        let step = crate::domain::entity::workflow_step::WorkflowStep {
            step_id: "step-1".to_string(),
            name: "Review".to_string(),
            step_type: "approval".to_string(),
            assignee_role: Some("manager".to_string()),
            timeout_hours: Some(24),
            on_approve: Some("step-2".to_string()),
            on_reject: Some("end".to_string()),
        };
        let resp = to_step_response(&step);
        assert_eq!(resp.step_id, "step-1");
        assert_eq!(resp.name, "Review");
        assert_eq!(resp.assignee_role.as_deref(), Some("manager"));
        assert_eq!(resp.timeout_hours, Some(24));
        assert_eq!(resp.on_approve.as_deref(), Some("step-2"));
        assert_eq!(resp.on_reject.as_deref(), Some("end"));
    }
}
