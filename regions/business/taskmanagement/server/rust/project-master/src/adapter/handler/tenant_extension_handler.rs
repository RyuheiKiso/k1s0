// テナント拡張 REST ハンドラ。
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;
use super::{error::map_domain_error, AppState};
use crate::domain::entity::tenant_project_extension::UpsertTenantExtension;

pub async fn get_extension(
    State(state): State<AppState>,
    Path((tenant_id, status_id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    state
        .manage_tenant_extensions_uc
        .get(&tenant_id, status_id)
        .await
        .map_err(map_domain_error)?
        .map(|ext| Json(ext))
        .ok_or_else(|| map_domain_error(anyhow::anyhow!("not found")))
}

pub async fn upsert_extension(
    State(state): State<AppState>,
    Path((tenant_id, status_id)): Path<(String, Uuid)>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let input = UpsertTenantExtension {
        tenant_id,
        status_definition_id: status_id,
        display_name_override: body.get("display_name_override").and_then(|v| v.as_str()).map(String::from),
        attributes_override: body.get("attributes_override").cloned(),
        is_enabled: body.get("is_enabled").and_then(|v| v.as_bool()),
    };
    state
        .manage_tenant_extensions_uc
        .upsert(&input)
        .await
        .map(|ext| Json(ext))
        .map_err(map_domain_error)
}

pub async fn delete_extension(
    State(state): State<AppState>,
    Path((tenant_id, status_id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    state
        .manage_tenant_extensions_uc
        .delete(&tenant_id, status_id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(map_domain_error)
}

pub async fn list_tenant_statuses(
    State(state): State<AppState>,
    Path((tenant_id, project_type_id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    state
        .manage_tenant_extensions_uc
        .list_merged(&tenant_id, project_type_id, false, 50, 0)
        .await
        .map(|(items, _)| Json(items))
        .map_err(map_domain_error)
}
