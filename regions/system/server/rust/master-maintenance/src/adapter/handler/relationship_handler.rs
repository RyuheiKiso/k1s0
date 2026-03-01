use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use crate::adapter::handler::AppState;
use crate::adapter::handler::error::AppError;

pub async fn list_relationships(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let relationships = state.manage_relationships_uc.list_relationships().await?;
    Ok(Json(serde_json::to_value(relationships).unwrap()))
}

pub async fn create_relationship(
    State(state): State<AppState>,
    Json(input): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let relationship = state
        .manage_relationships_uc
        .create_relationship(&input, "system")
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(relationship).unwrap()),
    ))
}

pub async fn update_relationship(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
    Json(input): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let relationship = state
        .manage_relationships_uc
        .update_relationship(id, &input)
        .await?;
    Ok(Json(serde_json::to_value(relationship).unwrap()))
}

pub async fn delete_relationship(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, AppError> {
    state.manage_relationships_uc.delete_relationship(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_related_records(
    State(state): State<AppState>,
    Path((name, id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let related = state
        .manage_relationships_uc
        .get_related_records(&name, &id)
        .await?;
    Ok(Json(related))
}
