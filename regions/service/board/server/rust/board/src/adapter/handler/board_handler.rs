// ボードカラム REST ハンドラー。
use crate::adapter::handler::AppState;
use crate::domain::entity::board_column::{
    BoardColumnFilter, DecrementColumnRequest, IncrementColumnRequest, UpdateWipLimitRequest,
};
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
        code: k1s0_server_common::ErrorCode::new("SVC_BOARD_ERROR"),
        message: e.to_string(),
    }
}

#[derive(Debug, Deserialize)]
pub struct ListBoardColumnsQuery {
    pub project_id: Option<Uuid>,
    pub status_code: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_board_columns(
    State(state): State<AppState>,
    Query(q): Query<ListBoardColumnsQuery>,
) -> Result<impl IntoResponse, ServiceError> {
    let filter = BoardColumnFilter {
        project_id: q.project_id,
        status_code: q.status_code,
        limit: q.limit,
        offset: q.offset,
    };
    let (cols, total) = state.list_board_columns_uc.execute(&filter).await.map_err(map_err)?;
    Ok(Json(serde_json::json!({ "columns": cols, "total": total })))
}

pub async fn get_board_column(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ServiceError> {
    let col = state
        .get_board_column_uc
        .execute(id)
        .await
        .map_err(map_err)?
        .ok_or_else(|| ServiceError::NotFound {
            code: k1s0_server_common::ErrorCode::new("SVC_BOARD_COLUMN_NOT_FOUND"),
            message: format!("BoardColumn '{}' not found", id),
        })?;
    Ok(Json(col))
}

pub async fn increment_column(
    State(state): State<AppState>,
    Json(req): Json<IncrementColumnRequest>,
) -> Result<impl IntoResponse, ServiceError> {
    let col = state.increment_column_uc.execute(&req).await.map_err(map_err)?;
    Ok((StatusCode::OK, Json(col)))
}

pub async fn decrement_column(
    State(state): State<AppState>,
    Json(req): Json<DecrementColumnRequest>,
) -> Result<impl IntoResponse, ServiceError> {
    let col = state.decrement_column_uc.execute(&req).await.map_err(map_err)?;
    Ok((StatusCode::OK, Json(col)))
}

pub async fn update_wip_limit(
    State(state): State<AppState>,
    Path(column_id): Path<Uuid>,
    Json(mut req): Json<UpdateWipLimitRequest>,
) -> Result<impl IntoResponse, ServiceError> {
    req.column_id = column_id;
    let col = state.update_wip_limit_uc.execute(&req).await.map_err(map_err)?;
    Ok(Json(col))
}
