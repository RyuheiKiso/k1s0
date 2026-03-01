use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use crate::adapter::handler::AppState;
use crate::adapter::handler::error::AppError;

#[derive(Debug, Deserialize)]
pub struct AuditLogQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

pub async fn list_table_audit_logs(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Query(query): Query<AuditLogQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (logs, total) = state.get_audit_logs_uc
        .get_table_logs(&name, query.page.unwrap_or(1), query.page_size.unwrap_or(20))
        .await?;
    Ok(Json(serde_json::json!({ "logs": logs, "total": total })))
}

pub async fn list_record_audit_logs(
    State(state): State<AppState>,
    Path((name, id)): Path<(String, String)>,
    Query(query): Query<AuditLogQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (logs, total) = state.get_audit_logs_uc
        .get_record_logs(&name, &id, query.page.unwrap_or(1), query.page_size.unwrap_or(20))
        .await?;
    Ok(Json(serde_json::json!({ "logs": logs, "total": total })))
}
