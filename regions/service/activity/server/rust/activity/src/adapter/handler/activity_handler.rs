use crate::adapter::handler::AppState;
use crate::domain::entity::activity::{ActivityFilter, CreateActivity};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use k1s0_server_common::ServiceError;
use serde::Deserialize;
use uuid::Uuid;

fn map_err(e: anyhow::Error) -> ServiceError {
    ServiceError::Internal {
        code: k1s0_server_common::ErrorCode::new("SVC_ACTIVITY_ERROR"),
        message: e.to_string(),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListActivitiesQuery {
    pub task_id: Option<Uuid>,
    pub actor_id: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_activities(
    State(state): State<AppState>,
    Query(q): Query<ListActivitiesQuery>,
) -> Result<impl IntoResponse, ServiceError> {
    let filter = ActivityFilter {
        task_id: q.task_id,
        actor_id: q.actor_id,
        status: q.status.as_deref().and_then(|s| s.parse().ok()),
        limit: q.limit,
        offset: q.offset,
    };
    let (activities, total) = state.list_activities_uc.execute(&filter).await.map_err(map_err)?;
    Ok(Json(serde_json::json!({ "activities": activities, "total": total })))
}

pub async fn get_activity(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ServiceError> {
    let activity = state
        .get_activity_uc
        .execute(id)
        .await
        .map_err(map_err)?
        .ok_or_else(|| ServiceError::NotFound {
            code: k1s0_server_common::ErrorCode::new("SVC_ACTIVITY_NOT_FOUND"),
            message: format!("Activity '{}' not found", id),
        })?;
    Ok(Json(activity))
}

pub async fn create_activity(
    State(state): State<AppState>,
    Json(input): Json<CreateActivity>,
) -> Result<impl IntoResponse, ServiceError> {
    let activity = state
        .create_activity_uc
        .execute(&input, "anonymous")
        .await
        .map_err(map_err)?;
    Ok((StatusCode::CREATED, Json(activity)))
}

pub async fn submit_activity(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ServiceError> {
    let activity = state
        .submit_activity_uc
        .execute(id, "anonymous")
        .await
        .map_err(map_err)?;
    Ok(Json(activity))
}

pub async fn approve_activity(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ServiceError> {
    let activity = state
        .approve_activity_uc
        .execute(id, "anonymous")
        .await
        .map_err(map_err)?;
    Ok(Json(activity))
}

pub async fn reject_activity(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ServiceError> {
    let activity = state
        .reject_activity_uc
        .execute(id, "anonymous", "no reason provided")
        .await
        .map_err(map_err)?;
    Ok(Json(activity))
}
