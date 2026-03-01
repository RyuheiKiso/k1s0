use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use crate::adapter::handler::AppState;
use crate::adapter::handler::error::AppError;

pub async fn list_display_configs(
    State(_state): State<AppState>,
    Path(_name): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    todo!("list_display_configs")
}

pub async fn get_display_config(
    State(_state): State<AppState>,
    Path((_name, _id)): Path<(String, uuid::Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    todo!("get_display_config")
}

pub async fn create_display_config(
    State(_state): State<AppState>,
    Path(_name): Path<String>,
    Json(_input): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    todo!("create_display_config")
}

pub async fn update_display_config(
    State(_state): State<AppState>,
    Path((_name, _id)): Path<(String, uuid::Uuid)>,
    Json(_input): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    todo!("update_display_config")
}

pub async fn delete_display_config(
    State(_state): State<AppState>,
    Path((_name, _id)): Path<(String, uuid::Uuid)>,
) -> Result<StatusCode, AppError> {
    todo!("delete_display_config")
}
