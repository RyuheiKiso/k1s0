use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use crate::adapter::handler::AppState;
use crate::adapter::handler::error::AppError;

#[derive(Debug, Deserialize)]
pub struct ListRulesQuery {
    pub table: Option<String>,
    pub rule_type: Option<String>,
    pub severity: Option<String>,
    pub timing: Option<String>,
}

pub async fn list_rules(
    State(state): State<AppState>,
    Query(query): Query<ListRulesQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rules = state.manage_rules_uc.list_rules(query.table.as_deref(), query.rule_type.as_deref(), query.severity.as_deref()).await?;
    Ok(Json(serde_json::to_value(rules).unwrap()))
}

pub async fn get_rule(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rule = state.manage_rules_uc.get_rule(id).await?
        .ok_or_else(|| AppError::not_found("SYS_MM_RULE_NOT_FOUND", "Rule not found"))?;
    Ok(Json(serde_json::to_value(rule).unwrap()))
}

pub async fn create_rule(
    State(state): State<AppState>,
    Json(input): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let rule = state.manage_rules_uc.create_rule(&input, "system").await?;
    Ok((StatusCode::CREATED, Json(serde_json::to_value(rule).unwrap())))
}

pub async fn update_rule(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
    Json(input): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let rule = state.manage_rules_uc.update_rule(id, &input).await?;
    Ok(Json(serde_json::to_value(rule).unwrap()))
}

pub async fn delete_rule(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, AppError> {
    state.manage_rules_uc.delete_rule(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn execute_rule(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = state.check_consistency_uc.execute_rule(id).await?;
    Ok(Json(serde_json::to_value(result).unwrap()))
}

pub async fn check_rules(
    State(state): State<AppState>,
    Json(input): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let table_name = input.get("table_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::bad_request("SYS_MM_VALIDATION_FAILED", "table_name is required"))?;
    let result = state.check_consistency_uc.check_all_rules(table_name).await?;
    Ok(Json(serde_json::to_value(result).unwrap()))
}
