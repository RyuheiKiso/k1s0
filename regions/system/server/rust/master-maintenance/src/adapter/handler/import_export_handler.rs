use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use crate::adapter::handler::AppState;
use crate::adapter::handler::error::AppError;

pub async fn import_records(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(data): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let job = state
        .import_export_uc
        .import_records(&name, &data, "system")
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(job).unwrap()),
    ))
}

pub async fn export_records(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = state.import_export_uc.export_records(&name).await?;
    Ok(Json(result))
}

pub async fn get_import_job(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let job = state
        .import_export_uc
        .get_import_job(id)
        .await?
        .ok_or_else(|| {
            AppError::not_found("SYS_MM_IMPORT_JOB_NOT_FOUND", "Import job not found")
        })?;
    Ok(Json(serde_json::to_value(job).unwrap()))
}
