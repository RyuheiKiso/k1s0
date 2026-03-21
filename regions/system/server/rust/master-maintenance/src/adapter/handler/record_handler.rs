use crate::adapter::handler::error::AppError;
use crate::adapter::handler::table_handler::DomainScopeQuery;
use crate::adapter::handler::{
    actor_from_claims, current_trace_id, publish_change_event, AppState,
};
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use k1s0_auth::Claims;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ListRecordsQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub sort: Option<String>,
    pub filter: Option<String>,
    pub search: Option<String>,
    pub columns: Option<String>,
    pub domain_scope: Option<String>,
}

/// レコード一覧ハンドラー。read 操作も認証必須（P0-2 対応）。
pub async fn list_records(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(name): Path<String>,
    Query(query): Query<ListRecordsQuery>,
) -> Result<impl IntoResponse, AppError> {
    // 認証トークンが存在しない場合は 401 を返す
    let _ = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    let result = state
        .crud_records_uc
        .list_records(
            &name,
            query.page.unwrap_or(1),
            query.page_size.unwrap_or(20),
            query.sort.as_deref(),
            query.filter.as_deref(),
            query.search.as_deref(),
            query.columns.as_deref(),
            query.domain_scope.as_deref(),
        )
        .await?;
    Ok(Json(serde_json::json!({
        "records": result.records,
        "total": result.total,
        "page": query.page.unwrap_or(1),
        "page_size": query.page_size.unwrap_or(20),
        "metadata": {
            "table_name": result.table_name,
            "display_name": result.display_name,
            "allow_create": result.allow_create,
            "allow_update": result.allow_update,
            "allow_delete": result.allow_delete
        }
    })))
}

/// レコード取得ハンドラー。read 操作も認証必須（P0-2 対応）。
pub async fn get_record(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path((name, id)): Path<(String, String)>,
    Query(ds_query): Query<DomainScopeQuery>,
) -> Result<impl IntoResponse, AppError> {
    // 認証トークンが存在しない場合は 401 を返す
    let _ = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    let record = state
        .crud_records_uc
        .get_record(&name, &id, ds_query.domain_scope.as_deref())
        .await?
        .ok_or_else(|| AppError::not_found("SYS_MM_RECORD_NOT_FOUND", "Record not found"))?;
    Ok(Json(record))
}

/// レコード作成ハンドラー。書き込み操作のため認証必須（P0-2 対応）。
pub async fn create_record(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(name): Path<String>,
    Query(ds_query): Query<DomainScopeQuery>,
    Json(data): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    // 認証トークンが存在しない場合は 401 を返す
    let claims_ext = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    let actor = actor_from_claims(Some(&claims_ext.0));
    let result = state
        .crud_records_uc
        .create_record(
            &name,
            &data,
            &actor,
            ds_query.domain_scope.as_deref(),
            current_trace_id(),
        )
        .await?;
    publish_change_event(
        &state,
        serde_json::json!({
            "event_type": "MASTER_MAINTENANCE_DATA_CHANGED",
            "resource_type": "record",
            "resource_id": result.record.get("id").and_then(|value| value.as_str()),
            "resource_name": name,
            "action": "created",
            "actor": actor,
            "after": result.record.clone(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }),
    )
    .await;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "data": result.record,
            "warnings": result.warnings,
        })),
    ))
}

/// レコード更新ハンドラー。書き込み操作のため認証必須（P0-2 対応）。
pub async fn update_record(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path((name, id)): Path<(String, String)>,
    Query(ds_query): Query<DomainScopeQuery>,
    Json(data): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    // 認証トークンが存在しない場合は 401 を返す
    let claims_ext = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    let actor = actor_from_claims(Some(&claims_ext.0));
    let result = state
        .crud_records_uc
        .update_record(
            &name,
            &id,
            &data,
            &actor,
            ds_query.domain_scope.as_deref(),
            current_trace_id(),
        )
        .await?;
    publish_change_event(
        &state,
        serde_json::json!({
            "event_type": "MASTER_MAINTENANCE_DATA_CHANGED",
            "resource_type": "record",
            "resource_id": id,
            "resource_name": name,
            "action": "updated",
            "actor": actor,
            "after": result.record.clone(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }),
    )
    .await;
    Ok(Json(serde_json::json!({
        "data": result.record,
        "warnings": result.warnings,
    })))
}

/// レコード削除ハンドラー。書き込み操作のため認証必須（P0-2 対応）。
pub async fn delete_record(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path((name, id)): Path<(String, String)>,
    Query(ds_query): Query<DomainScopeQuery>,
) -> Result<StatusCode, AppError> {
    // 認証トークンが存在しない場合は 401 を返す
    let claims_ext = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    let actor = actor_from_claims(Some(&claims_ext.0));
    state
        .crud_records_uc
        .delete_record(
            &name,
            &id,
            &actor,
            ds_query.domain_scope.as_deref(),
            current_trace_id(),
        )
        .await?;
    publish_change_event(
        &state,
        serde_json::json!({
            "event_type": "MASTER_MAINTENANCE_DATA_CHANGED",
            "resource_type": "record",
            "resource_id": id,
            "resource_name": name,
            "action": "deleted",
            "actor": actor,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }),
    )
    .await;
    Ok(StatusCode::NO_CONTENT)
}
