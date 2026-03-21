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

/// ルール一覧ハンドラー。read 操作も認証必須（P0-2 対応）。
pub async fn list_rules(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Query(query): Query<ListRulesQuery>,
) -> Result<impl IntoResponse, AppError> {
    // 認証トークンが存在しない場合は 401 を返す
    let _guard = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
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

/// ルール取得ハンドラー。read 操作も認証必須（P0-2 対応）。
pub async fn get_rule(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(id): Path<uuid::Uuid>,
) -> Result<impl IntoResponse, AppError> {
    // 認証トークンが存在しない場合は 401 を返す
    let _guard = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    let rule = state
        .manage_rules_uc
        .get_rule(id)
        .await?
        .ok_or_else(|| AppError::not_found("SYS_MM_RULE_NOT_FOUND", "Rule not found"))?;
    Ok(Json(rule))
}

/// ルール作成ハンドラー。書き込み操作のため認証必須（P0-2 対応）。
pub async fn create_rule(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Json(input): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    // 認証トークンが存在しない場合は 401 を返す
    let claims_ext = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    let actor = actor_from_claims(Some(&claims_ext.0));
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

/// ルール更新ハンドラー。書き込み操作のため認証必須（P0-2 対応）。
pub async fn update_rule(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(id): Path<uuid::Uuid>,
    Json(input): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    // 認証トークンが存在しない場合は 401 を返す
    let _guard = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    let rule = state.manage_rules_uc.update_rule(id, &input, None).await?;
    Ok(Json(rule))
}

/// ルール削除ハンドラー。書き込み操作のため認証必須（P0-2 対応）。
pub async fn delete_rule(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, AppError> {
    // 認証トークンが存在しない場合は 401 を返す
    let _guard = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    state.manage_rules_uc.delete_rule(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// ルール実行ハンドラー。実行操作のため認証必須（P0-2 対応）。
pub async fn execute_rule(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(id): Path<uuid::Uuid>,
) -> Result<impl IntoResponse, AppError> {
    // 認証トークンが存在しない場合は 401 を返す
    let _guard = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    let result = state.check_consistency_uc.execute_rule(id, None).await?;
    Ok(Json(result))
}

/// ルール一括チェックハンドラー。実行操作のため認証必須（P0-2 対応）。
pub async fn check_rules(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Json(input): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    // 認証トークンが存在しない場合は 401 を返す
    let _guard = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
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
