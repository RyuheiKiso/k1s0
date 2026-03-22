// タスク REST ハンドラー。
use crate::adapter::handler::AppState;
use crate::domain::entity::task::{CreateTask, TaskFilter, UpdateTaskStatus};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
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
    Query(q): Query<ListTasksQuery>,
) -> Result<impl IntoResponse, ServiceError> {
    let filter = TaskFilter {
        project_id: q.project_id,
        assignee_id: q.assignee_id,
        status: q.status.as_deref().and_then(|s| s.parse().ok()),
        limit: q.limit,
        offset: q.offset,
    };
    let (tasks, total) = state.list_tasks_uc.execute(&filter).await.map_err(map_err)?;
    Ok(Json(serde_json::json!({ "tasks": tasks, "total": total })))
}

pub async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ServiceError> {
    let task = state
        .get_task_uc
        .execute(id)
        .await
        .map_err(map_err)?
        .ok_or_else(|| ServiceError::NotFound {
            code: k1s0_server_common::ErrorCode::new("SVC_TASK_NOT_FOUND"),
            message: format!("Task '{}' not found", id),
        })?;
    Ok(Json(task))
}

pub async fn create_task(
    State(state): State<AppState>,
    Json(input): Json<CreateTask>,
) -> Result<impl IntoResponse, ServiceError> {
    let task = state
        .create_task_uc
        .execute(&input, "anonymous")
        .await
        .map_err(map_err)?;
    Ok((StatusCode::CREATED, Json(task)))
}

pub async fn update_task_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateTaskStatus>,
) -> Result<impl IntoResponse, ServiceError> {
    let task = state
        .update_task_status_uc
        .execute(id, &input, "anonymous")
        .await
        .map_err(map_err)?;
    Ok(Json(task))
}

pub async fn get_checklist(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ServiceError> {
    let items = state.get_task_uc.get_checklist(id).await.map_err(map_err)?;
    Ok(Json(serde_json::json!({ "checklist": items })))
}
