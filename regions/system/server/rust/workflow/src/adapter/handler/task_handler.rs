// タスク管理ハンドラ
// タスクの一覧取得・期限超過チェック・承認・却下・再割り当て操作を提供する
// RUST-CRIT-001 対応: Claims から tenant_id を取得してテナント境界を適用する

use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use k1s0_auth::Claims;

use crate::usecase::approve_task::{ApproveTaskError, ApproveTaskInput};
use crate::usecase::check_overdue_tasks::CheckOverdueTasksError;
use crate::usecase::list_tasks::{ListTasksError, ListTasksInput};
use crate::usecase::reassign_task::{ReassignTaskError, ReassignTaskInput};
use crate::usecase::reject_task::{RejectTaskError, RejectTaskInput};

use super::dto::{
    error_json, AppState, ApproveTaskRequest, ListTasksQuery, ReassignTaskRequest,
    RejectTaskRequest,
};

/// Claims が存在する場合は `tenant_id` を返し、存在しない場合は "system" を返す
fn tenant_id_from_claims(claims: Option<&Claims>) -> String {
    claims.map_or_else(|| "system".to_string(), |c| c.tenant_id().to_string())
}

/// GET /api/v1/tasks
/// フィルタ条件に基づいてタスク一覧を取得する
pub async fn list_tasks(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Query(query): Query<ListTasksQuery>,
) -> impl IntoResponse {
    // RLS テナント分離のため Claims から tenant_id を取得する
    let tenant_id = tenant_id_from_claims(claims.as_ref().map(|e| &e.0));
    // クエリパラメータからユースケース入力を組み立てる
    let input = ListTasksInput {
        tenant_id,
        assignee_id: query.assignee_id,
        status: query.status,
        instance_id: query.instance_id,
        overdue_only: query.overdue_only,
        page: query.page,
        page_size: query.page_size,
    };

    match state.list_tasks_uc.execute(&input).await {
        Ok(output) => {
            // 各タスクをJSON値に変換し、期限超過フラグを算出
            let tasks: Vec<serde_json::Value> = output
                .tasks
                .into_iter()
                .map(|t| {
                    // 期限超過判定: 期限切れかつステータスがpendingまたはassignedの場合
                    let is_overdue = t.due_at.is_some_and(|d| {
                        d < chrono::Utc::now() && (t.status == "pending" || t.status == "assigned")
                    });
                    serde_json::json!({
                        "id": t.id,
                        "instance_id": t.instance_id,
                        "step_id": t.step_id,
                        "step_name": t.step_name,
                        "assignee_id": t.assignee_id,
                        "status": t.status,
                        "is_overdue": is_overdue,
                        "due_at": t.due_at.map(|d| d.to_rfc3339()),
                        "created_at": t.created_at.to_rfc3339(),
                        "updated_at": t.updated_at.to_rfc3339(),
                    })
                })
                .collect();
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "tasks": tasks,
                    "pagination": {
                        "total_count": output.total_count,
                        "page": output.page,
                        "page_size": output.page_size,
                        "has_next": output.has_next
                    }
                })),
            )
                .into_response()
        }
        // 内部エラー
        Err(ListTasksError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_json("SYS_WORKFLOW_INTERNAL_ERROR", &msg)),
        )
            .into_response(),
    }
}

/// POST /internal/tasks/check-overdue
/// 期限超過タスクをチェックしてイベントを発行する（内部API）
pub async fn check_overdue_tasks(State(state): State<AppState>) -> impl IntoResponse {
    match state.check_overdue_tasks_uc.execute().await {
        Ok(output) => {
            // 期限超過タスク情報をJSON値に変換
            let tasks: Vec<serde_json::Value> = output
                .overdue_tasks
                .into_iter()
                .map(|task| {
                    serde_json::json!({
                        "task_id": task.id,
                        "instance_id": task.instance_id,
                        "assignee_id": task.assignee_id,
                        "due_at": task.due_at.map(|d| d.to_rfc3339()),
                    })
                })
                .collect();

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "overdue_count": output.count,
                    "published_count": output.published_count,
                    "tasks": tasks
                })),
            )
                .into_response()
        }
        // 内部エラー
        Err(CheckOverdueTasksError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_json("SYS_WORKFLOW_INTERNAL_ERROR", &msg)),
        )
            .into_response(),
    }
}

/// POST /api/v1/tasks/:id/approve
/// タスクを承認し、次のステップへ進行させる
pub async fn approve_task(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(id): Path<String>,
    Json(req): Json<ApproveTaskRequest>,
) -> impl IntoResponse {
    // RLS テナント分離のため Claims から tenant_id を取得する
    let tenant_id = tenant_id_from_claims(claims.as_ref().map(|e| &e.0));
    let input = ApproveTaskInput {
        tenant_id,
        task_id: id.clone(),
        actor_id: req.actor_user_id,
        comment: req.comment,
    };

    match state.approve_task_uc.execute(&input).await {
        Ok(output) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "task_id": output.task.id,
                "status": output.task.status,
                "decided_at": output.task.decided_at.map(|t| t.to_rfc3339()),
                "instance_status": output.instance_status,
                "next_task_id": output.next_task.map(|t| t.id)
            })),
        )
            .into_response(),
        // タスクが見つからない場合
        Err(ApproveTaskError::TaskNotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(error_json(
                "SYS_WORKFLOW_TASK_NOT_FOUND",
                &format!("task not found: {id}"),
            )),
        )
            .into_response(),
        // 承認できないステータスの場合
        Err(ApproveTaskError::InvalidStatus(status)) => (
            StatusCode::CONFLICT,
            Json(error_json(
                "SYS_WORKFLOW_TASK_INVALID_STATUS",
                &format!("invalid task status: {status}"),
            )),
        )
            .into_response(),
        // 関連インスタンスが見つからない場合
        Err(ApproveTaskError::InstanceNotFound(inst_id)) => (
            StatusCode::NOT_FOUND,
            Json(error_json(
                "SYS_WORKFLOW_INSTANCE_NOT_FOUND",
                &format!("instance not found: {inst_id}"),
            )),
        )
            .into_response(),
        // ワークフロー定義が見つからない場合
        Err(ApproveTaskError::DefinitionNotFound(def_id)) => (
            StatusCode::NOT_FOUND,
            Json(error_json(
                "SYS_WORKFLOW_DEFINITION_NOT_FOUND",
                &format!("definition not found: {def_id}"),
            )),
        )
            .into_response(),
        // 内部エラー
        Err(ApproveTaskError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_json("SYS_WORKFLOW_INTERNAL_ERROR", &msg)),
        )
            .into_response(),
    }
}

/// POST /api/v1/tasks/:id/reject
/// タスクを却下し、ワークフローを分岐させる
pub async fn reject_task(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(id): Path<String>,
    Json(req): Json<RejectTaskRequest>,
) -> impl IntoResponse {
    // RLS テナント分離のため Claims から tenant_id を取得する
    let tenant_id = tenant_id_from_claims(claims.as_ref().map(|e| &e.0));
    let input = RejectTaskInput {
        tenant_id,
        task_id: id.clone(),
        actor_id: req.actor_user_id,
        comment: req.comment,
    };

    match state.reject_task_uc.execute(&input).await {
        Ok(output) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "task_id": output.task.id,
                "status": output.task.status,
                "decided_at": output.task.decided_at.map(|t| t.to_rfc3339()),
                "instance_status": output.instance_status,
                "next_task_id": output.next_task.map(|t| t.id)
            })),
        )
            .into_response(),
        // タスクが見つからない場合
        Err(RejectTaskError::TaskNotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(error_json(
                "SYS_WORKFLOW_TASK_NOT_FOUND",
                &format!("task not found: {id}"),
            )),
        )
            .into_response(),
        // 却下できないステータスの場合
        Err(RejectTaskError::InvalidStatus(status)) => (
            StatusCode::CONFLICT,
            Json(error_json(
                "SYS_WORKFLOW_TASK_INVALID_STATUS",
                &format!("invalid task status: {status}"),
            )),
        )
            .into_response(),
        // 関連インスタンスが見つからない場合
        Err(RejectTaskError::InstanceNotFound(inst_id)) => (
            StatusCode::NOT_FOUND,
            Json(error_json(
                "SYS_WORKFLOW_INSTANCE_NOT_FOUND",
                &format!("instance not found: {inst_id}"),
            )),
        )
            .into_response(),
        // ワークフロー定義が見つからない場合
        Err(RejectTaskError::DefinitionNotFound(def_id)) => (
            StatusCode::NOT_FOUND,
            Json(error_json(
                "SYS_WORKFLOW_DEFINITION_NOT_FOUND",
                &format!("definition not found: {def_id}"),
            )),
        )
            .into_response(),
        // 内部エラー
        Err(RejectTaskError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_json("SYS_WORKFLOW_INTERNAL_ERROR", &msg)),
        )
            .into_response(),
    }
}

/// POST /api/v1/tasks/:id/reassign
/// タスクを別のユーザーに再割り当てする
pub async fn reassign_task(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(id): Path<String>,
    Json(req): Json<ReassignTaskRequest>,
) -> impl IntoResponse {
    // RLS テナント分離のため Claims から tenant_id を取得する
    let tenant_id = tenant_id_from_claims(claims.as_ref().map(|e| &e.0));
    let input = ReassignTaskInput {
        tenant_id,
        task_id: id.clone(),
        new_assignee_id: req.new_assignee_id,
        reason: req.reason,
        actor_id: req.actor_user_id,
    };

    match state.reassign_task_uc.execute(&input).await {
        Ok(output) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "task_id": output.task.id,
                "new_assignee_id": output.task.assignee_id,
                "previous_assignee_id": output.previous_assignee_id,
                "reassigned_at": output.task.updated_at.to_rfc3339(),
                "message": "task reassigned"
            })),
        )
            .into_response(),
        // タスクが見つからない場合
        Err(ReassignTaskError::TaskNotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(error_json(
                "SYS_WORKFLOW_TASK_NOT_FOUND",
                &format!("task not found: {id}"),
            )),
        )
            .into_response(),
        // 再割り当てできないステータスの場合
        Err(ReassignTaskError::InvalidStatus(status)) => (
            StatusCode::CONFLICT,
            Json(error_json(
                "SYS_WORKFLOW_TASK_REASSIGN_INVALID_STATUS",
                &format!("invalid task status for reassignment: {status}"),
            )),
        )
            .into_response(),
        // 内部エラー
        Err(ReassignTaskError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_json("SYS_WORKFLOW_INTERNAL_ERROR", &msg)),
        )
            .into_response(),
    }
}
