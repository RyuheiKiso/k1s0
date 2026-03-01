use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use crate::adapter::handler::AppState;
use crate::adapter::handler::error::AppError;

pub async fn list_display_configs(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let configs = state
        .manage_display_configs_uc
        .list_display_configs(&name)
        .await?;
    Ok(Json(serde_json::to_value(configs).unwrap()))
}

pub async fn get_display_config(
    State(state): State<AppState>,
    Path((_name, id)): Path<(String, uuid::Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let config = state
        .manage_display_configs_uc
        .get_display_config(id)
        .await?
        .ok_or_else(|| {
            AppError::not_found("SYS_MM_DISPLAY_CONFIG_NOT_FOUND", "Display config not found")
        })?;
    Ok(Json(serde_json::to_value(config).unwrap()))
}

pub async fn create_display_config(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(input): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let config = state
        .manage_display_configs_uc
        .create_display_config(&name, &input, "system")
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(config).unwrap()),
    ))
}

pub async fn update_display_config(
    State(state): State<AppState>,
    Path((_name, id)): Path<(String, uuid::Uuid)>,
    Json(input): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let config = state
        .manage_display_configs_uc
        .update_display_config(id, &input)
        .await?;
    Ok(Json(serde_json::to_value(config).unwrap()))
}

pub async fn delete_display_config(
    State(state): State<AppState>,
    Path((_name, id)): Path<(String, uuid::Uuid)>,
) -> Result<StatusCode, AppError> {
    state
        .manage_display_configs_uc
        .delete_display_config(id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
