use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use crate::adapter::handler::AppState;
use crate::adapter::handler::error::AppError;
use crate::domain::entity::table_definition::{CreateTableDefinition, UpdateTableDefinition};

#[derive(Debug, Deserialize)]
pub struct ListTablesQuery {
    pub category: Option<String>,
    pub active_only: Option<bool>,
}

pub async fn healthz() -> StatusCode {
    StatusCode::OK
}

pub async fn readyz(State(_state): State<AppState>) -> StatusCode {
    StatusCode::OK
}

pub async fn metrics_handler() -> String {
    // TODO: Prometheus metrics
    String::new()
}

pub async fn list_tables(
    State(state): State<AppState>,
    Query(query): Query<ListTablesQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let tables = state.manage_tables_uc
        .list_tables(query.category.as_deref(), query.active_only.unwrap_or(false))
        .await?;
    Ok(Json(serde_json::to_value(tables).unwrap()))
}

pub async fn get_table(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let table = state.manage_tables_uc
        .get_table(&name)
        .await?
        .ok_or_else(|| AppError::not_found("SYS_MM_TABLE_NOT_FOUND", &format!("Table '{}' not found", name)))?;
    Ok(Json(serde_json::to_value(table).unwrap()))
}

pub async fn create_table(
    State(state): State<AppState>,
    Json(input): Json<CreateTableDefinition>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let table = state.manage_tables_uc
        .create_table(&input, "system") // TODO: extract user from claims
        .await?;
    Ok((StatusCode::CREATED, Json(serde_json::to_value(table).unwrap())))
}

pub async fn update_table(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(input): Json<UpdateTableDefinition>,
) -> Result<Json<serde_json::Value>, AppError> {
    let table = state.manage_tables_uc
        .update_table(&name, &input)
        .await?;
    Ok(Json(serde_json::to_value(table).unwrap()))
}

pub async fn delete_table(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<StatusCode, AppError> {
    state.manage_tables_uc.delete_table(&name).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_table_schema(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let schema = state.manage_tables_uc.get_table_schema(&name).await?;
    Ok(Json(schema))
}

pub async fn list_columns(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let columns = state.manage_columns_uc.list_columns(&name).await?;
    Ok(Json(serde_json::to_value(columns).unwrap()))
}

pub async fn create_columns(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(input): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let columns = state.manage_columns_uc.create_columns(&name, &input).await?;
    Ok((StatusCode::CREATED, Json(serde_json::to_value(columns).unwrap())))
}

pub async fn update_column(
    State(state): State<AppState>,
    Path((name, column)): Path<(String, String)>,
    Json(input): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let col = state.manage_columns_uc.update_column(&name, &column, &input).await?;
    Ok(Json(serde_json::to_value(col).unwrap()))
}

pub async fn delete_column(
    State(state): State<AppState>,
    Path((name, column)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    state.manage_columns_uc.delete_column(&name, &column).await?;
    Ok(StatusCode::NO_CONTENT)
}
