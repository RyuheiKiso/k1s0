// タスク REST ハンドラー。
// Claims 拡張から認証ユーザー ID を取得してユースケースに渡す。
// RLS テナント分離のため Claims::tenant_id() メソッドを使用して tenant_id を取得する。
// Keycloak の tenant_id Protocol Mapper で設定されたカスタムクレームを優先する。
// Claims が存在しない（未認証）場合は 401 Unauthorized を返す。
use crate::adapter::handler::AppState;
use crate::domain::entity::task::{AddChecklistItem, CreateTask, TaskFilter, UpdateChecklistItem, UpdateTask, UpdateTaskStatus};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use k1s0_auth::claims::actor_from_claims;
use k1s0_auth::Claims;
use k1s0_server_common::ServiceError;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ListTasksQuery {
    pub project_id: Option<Uuid>,
    pub assignee_id: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

fn map_err(e: anyhow::Error) -> ServiceError {
    ServiceError::Internal {
        code: k1s0_server_common::ErrorCode::new("SVC_TASK_ERROR"),
        message: e.to_string(),
    }
}

pub async fn list_tasks(
    State(state): State<AppState>,
    claims: Option<axum::extract::Extension<Claims>>,
    Query(q): Query<ListTasksQuery>,
) -> Result<impl IntoResponse, ServiceError> {
    // Claims が存在しない場合は未認証として 401 を返す
    let claims_inner = claims
        .as_ref()
        .ok_or_else(|| ServiceError::unauthorized("TASK", "認証が必要です"))?;
    // RLS テナント分離のため Claims から tenant_id を取得する
    let tenant_id = claims_inner.0.tenant_id();
    // MED-14 監査対応: status パース失敗時に None で無視するのではなく 400 Bad Request を返す。
    // 無効なステータス値（例: "invalid_status"）でも検索が実行されると全件ヒットする可能性があり
    // ユーザーの意図と異なる結果を返すためエラーとして扱う。
    let status = match q.status.as_deref() {
        None => None,
        Some(s) => Some(s.parse::<crate::domain::entity::task::TaskStatus>().map_err(|e| {
            ServiceError::bad_request("TASK", format!("無効なステータス値です: {}", e))
        })?),
    };
    let filter = TaskFilter {
        project_id: q.project_id,
        assignee_id: q.assignee_id,
        status,
        limit: q.limit,
        offset: q.offset,
    };
    let (tasks, total) = state.list_tasks_uc.execute(tenant_id, &filter).await.map_err(map_err)?;
    Ok(Json(serde_json::json!({ "tasks": tasks, "total": total })))
}

pub async fn get_task(
    State(state): State<AppState>,
    claims: Option<axum::extract::Extension<Claims>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ServiceError> {
    // Claims が存在しない場合は未認証として 401 を返す
    let claims_inner = claims
        .as_ref()
        .ok_or_else(|| ServiceError::unauthorized("TASK", "認証が必要です"))?;
    // RLS テナント分離のため Claims から tenant_id を取得する
    let tenant_id = claims_inner.0.tenant_id();
    let task = state
        .get_task_uc
        .execute(tenant_id, id)
        .await
        .map_err(map_err)?
        .ok_or_else(|| ServiceError::NotFound {
            code: k1s0_server_common::ErrorCode::new("SVC_TASK_NOT_FOUND"),
            message: format!("Task '{}' not found", id),
        })?;
    Ok(Json(task))
}

// タスク作成: リクエスト拡張から Claims を取得し、actor を created_by および reporter_id として使用する
pub async fn create_task(
    State(state): State<AppState>,
    claims: Option<axum::extract::Extension<Claims>>,
    Json(mut input): Json<CreateTask>,
) -> Result<impl IntoResponse, ServiceError> {
    // Claims が存在しない場合は未認証として 401 を返す
    let claims_inner = claims
        .as_ref()
        .ok_or_else(|| ServiceError::unauthorized("TASK", "認証が必要です"))?;
    // RLS テナント分離のため Claims から tenant_id を取得する
    let tenant_id = claims_inner.0.tenant_id();
    // JWT sub/username を actor として使用する
    let actor = actor_from_claims(Some(&claims_inner.0));
    // reporter_id が未設定の場合、actor（リクエスト送信者）を reporter_id として設定する
    if input.reporter_id.is_none() {
        input.reporter_id = Some(actor.clone());
    }
    let task = state
        .create_task_uc
        .execute(tenant_id, &input, &actor)
        .await
        .map_err(map_err)?;
    Ok((StatusCode::CREATED, Json(task)))
}

// タスクステータス更新: リクエスト拡張から Claims を取得し、actor を updated_by として使用する
pub async fn update_task_status(
    State(state): State<AppState>,
    claims: Option<axum::extract::Extension<Claims>>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateTaskStatus>,
) -> Result<impl IntoResponse, ServiceError> {
    // Claims が存在しない場合は未認証として 401 を返す
    let claims_inner = claims
        .as_ref()
        .ok_or_else(|| ServiceError::unauthorized("TASK", "認証が必要です"))?;
    // RLS テナント分離のため Claims から tenant_id を取得する
    let tenant_id = claims_inner.0.tenant_id();
    // JWT sub/username を actor として使用する
    let actor = actor_from_claims(Some(&claims_inner.0));
    let task = state
        .update_task_status_uc
        .execute(tenant_id, id, &input, &actor)
        .await
        .map_err(map_err)?;
    Ok(Json(task))
}

pub async fn get_checklist(
    State(state): State<AppState>,
    claims: Option<axum::extract::Extension<Claims>>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ServiceError> {
    // Claims が存在しない場合は未認証として 401 を返す
    let claims_inner = claims
        .as_ref()
        .ok_or_else(|| ServiceError::unauthorized("TASK", "認証が必要です"))?;
    // RLS テナント分離のため Claims から tenant_id を取得する
    let tenant_id = claims_inner.0.tenant_id();
    let items = state.get_task_uc.get_checklist(tenant_id, id).await.map_err(map_err)?;
    Ok(Json(serde_json::json!({ "checklist": items })))
}

// タスク更新: リクエスト本体から UpdateTask を受け取り部分更新する
pub async fn update_task(
    State(state): State<AppState>,
    claims: Option<axum::extract::Extension<Claims>>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateTask>,
) -> Result<impl IntoResponse, ServiceError> {
    // Claims が存在しない場合は未認証として 401 を返す
    let claims_inner = claims
        .as_ref()
        .ok_or_else(|| ServiceError::unauthorized("TASK", "認証が必要です"))?;
    // RLS テナント分離のため Claims から tenant_id を取得する
    let tenant_id = claims_inner.0.tenant_id();
    // JWT sub/username を actor として使用する
    let actor = actor_from_claims(Some(&claims_inner.0));
    let task = state
        .update_task_uc
        .execute(tenant_id, id, &input, &actor)
        .await
        .map_err(map_err)?;
    Ok(Json(task))
}

// チェックリスト項目追加: リクエスト本体から AddChecklistItem を受け取り追加する
pub async fn create_checklist_item(
    State(state): State<AppState>,
    claims: Option<axum::extract::Extension<Claims>>,
    Path(task_id): Path<Uuid>,
    Json(input): Json<AddChecklistItem>,
) -> Result<impl IntoResponse, ServiceError> {
    // Claims が存在しない場合は未認証として 401 を返す
    let claims_inner = claims
        .as_ref()
        .ok_or_else(|| ServiceError::unauthorized("TASK", "認証が必要です"))?;
    // RLS テナント分離のため Claims から tenant_id を取得する
    let tenant_id = claims_inner.0.tenant_id();
    let item = state
        .create_checklist_item_uc
        .execute(tenant_id, task_id, &input)
        .await
        .map_err(map_err)?;
    Ok((StatusCode::CREATED, Json(item)))
}

// チェックリスト項目更新: リクエスト本体から UpdateChecklistItem を受け取り更新する
pub async fn update_checklist_item(
    State(state): State<AppState>,
    claims: Option<axum::extract::Extension<Claims>>,
    Path((task_id, item_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<UpdateChecklistItem>,
) -> Result<impl IntoResponse, ServiceError> {
    // Claims が存在しない場合は未認証として 401 を返す
    let claims_inner = claims
        .as_ref()
        .ok_or_else(|| ServiceError::unauthorized("TASK", "認証が必要です"))?;
    // RLS テナント分離のため Claims から tenant_id を取得する
    let tenant_id = claims_inner.0.tenant_id();
    let item = state
        .update_checklist_item_uc
        .execute(tenant_id, task_id, item_id, &input)
        .await
        .map_err(map_err)?;
    Ok(Json(item))
}

// チェックリスト項目削除: パスパラメータの item_id を削除する
pub async fn delete_checklist_item(
    State(state): State<AppState>,
    claims: Option<axum::extract::Extension<Claims>>,
    Path((task_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, ServiceError> {
    // Claims が存在しない場合は未認証として 401 を返す
    let claims_inner = claims
        .as_ref()
        .ok_or_else(|| ServiceError::unauthorized("TASK", "認証が必要です"))?;
    // RLS テナント分離のため Claims から tenant_id を取得する
    let tenant_id = claims_inner.0.tenant_id();
    state
        .delete_checklist_item_uc
        .execute(tenant_id, task_id, item_id)
        .await
        .map_err(map_err)?;
    Ok(StatusCode::NO_CONTENT)
}
