use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use k1s0_auth::Claims;
use serde::Deserialize;

use crate::adapter::handler::error::AppError;
use crate::adapter::handler::{actor_from_claims, publish_change_event, AppState};

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
) -> Result<impl IntoResponse, AppError> {
    let rules = state
        .manage_rules_uc
        .list_rules(
            query.table.as_deref(),
            query.rule_type.as_deref(),
            query.severity.as_deref(),
            None,
        )
        .await?;
    Ok(Json(rules))
}

pub async fn get_rule(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let rule = state
        .manage_rules_uc
        .get_rule(id)
        .await?
        .ok_or_else(|| AppError::not_found("SYS_MM_RULE_NOT_FOUND", "Rule not found"))?;
    Ok(Json(rule))
}

pub async fn create_rule(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Json(input): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let actor = actor_from_claims(claims.as_ref().map(|Extension(claims)| claims));
    let rule = state
        .manage_rules_uc
        .create_rule(&input, &actor, None)
        .await?;
    publish_change_event(
        &state,
        serde_json::json!({
            "event_type": "MASTER_MAINTENANCE_DATA_CHANGED",
            "resource_type": "rule",
            "resource_id": rule.id,
            "resource_name": rule.name,
            "action": "created",
            "actor": actor,
            "after": rule.clone(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }),
    )
    .await;
    Ok((StatusCode::CREATED, Json(rule)))
}

pub async fn update_rule(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
    Json(input): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let rule = state.manage_rules_uc.update_rule(id, &input, None).await?;
    Ok(Json(rule))
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
) -> Result<impl IntoResponse, AppError> {
    let result = state.check_consistency_uc.execute_rule(id, None).await?;
    Ok(Json(result))
}

pub async fn check_rules(
    State(state): State<AppState>,
    Json(input): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let table_name = input
        .get("table_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            AppError::bad_request("SYS_MM_VALIDATION_ERROR", "table_name is required")
        })?;
    let result = state
        .check_consistency_uc
        .check_all_rules(table_name, None)
        .await?;
    Ok(Json(result))
}
