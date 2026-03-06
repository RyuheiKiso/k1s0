use crate::adapter::handler::error::AppError;
use crate::adapter::handler::{actor_from_claims, publish_change_event, AppState};
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
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
}

pub async fn list_records(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Query(query): Query<ListRecordsQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
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

pub async fn get_record(
    State(state): State<AppState>,
    Path((name, id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let record = state
        .crud_records_uc
        .get_record(&name, &id)
        .await?
        .ok_or_else(|| AppError::not_found("SYS_MM_RECORD_NOT_FOUND", "Record not found"))?;
    Ok(Json(record))
}

pub async fn create_record(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(name): Path<String>,
    Json(data): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let actor = actor_from_claims(claims.as_ref().map(|Extension(claims)| claims));
    let result = state
        .crud_records_uc
        .create_record(&name, &data, &actor)
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

pub async fn update_record(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path((name, id)): Path<(String, String)>,
    Json(data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let actor = actor_from_claims(claims.as_ref().map(|Extension(claims)| claims));
    let result = state
        .crud_records_uc
        .update_record(&name, &id, &data, &actor)
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

pub async fn delete_record(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path((name, id)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    let actor = actor_from_claims(claims.as_ref().map(|Extension(claims)| claims));
    state
        .crud_records_uc
        .delete_record(&name, &id, &actor)
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
