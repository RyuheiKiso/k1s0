use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use crate::adapter::handler::AppState;
use crate::adapter::handler::error::AppError;

pub async fn import_records(
    State(_state): State<AppState>,
    Path(_name): Path<String>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    todo!("import_records")
}

pub async fn export_records(
    State(_state): State<AppState>,
    Path(_name): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    todo!("export_records")
}

pub async fn get_import_job(
    State(_state): State<AppState>,
    Path(_id): Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    todo!("get_import_job")
}
