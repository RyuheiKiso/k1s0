use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use crate::adapter::handler::AppState;
use crate::adapter::handler::error::AppError;

pub async fn list_relationships(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    todo!("list_relationships")
}

pub async fn create_relationship(
    State(_state): State<AppState>,
    Json(_input): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    todo!("create_relationship")
}

pub async fn update_relationship(
    State(_state): State<AppState>,
    Path(_id): Path<uuid::Uuid>,
    Json(_input): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    todo!("update_relationship")
}

pub async fn delete_relationship(
    State(_state): State<AppState>,
    Path(_id): Path<uuid::Uuid>,
) -> Result<StatusCode, AppError> {
    todo!("delete_relationship")
}

pub async fn get_related_records(
    State(_state): State<AppState>,
    Path((_name, _id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, AppError> {
    todo!("get_related_records")
}
