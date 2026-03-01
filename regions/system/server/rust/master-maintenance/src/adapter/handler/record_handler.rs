use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use crate::adapter::handler::AppState;
use crate::adapter::handler::error::AppError;

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
    let (records, total) = state.crud_records_uc
        .list_records(
            &name,
            query.page.unwrap_or(1),
            query.page_size.unwrap_or(20),
            query.sort.as_deref(),
            query.filter.as_deref(),
            query.search.as_deref(),
        )
        .await?;
    Ok(Json(serde_json::json!({
        "records": records,
        "total": total,
        "page": query.page.unwrap_or(1),
        "page_size": query.page_size.unwrap_or(20),
    })))
}

pub async fn get_record(
    State(state): State<AppState>,
    Path((name, id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let record = state.crud_records_uc
        .get_record(&name, &id)
        .await?
        .ok_or_else(|| AppError::not_found("SYS_MM_RECORD_NOT_FOUND", "Record not found"))?;
    Ok(Json(record))
}

pub async fn create_record(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(data): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let record = state.crud_records_uc
        .create_record(&name, &data, "system") // TODO: extract from claims
        .await?;
    Ok((StatusCode::CREATED, Json(record)))
}

pub async fn update_record(
    State(state): State<AppState>,
    Path((name, id)): Path<(String, String)>,
    Json(data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let record = state.crud_records_uc
        .update_record(&name, &id, &data, "system")
        .await?;
    Ok(Json(record))
}

pub async fn delete_record(
    State(state): State<AppState>,
    Path((name, id)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    state.crud_records_uc.delete_record(&name, &id, "system").await?;
    Ok(StatusCode::NO_CONTENT)
}
