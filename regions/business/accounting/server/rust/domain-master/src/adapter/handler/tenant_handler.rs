use crate::adapter::handler::error::from_anyhow;
use crate::adapter::handler::AppState;
use crate::domain::entity::tenant_master_extension::UpsertTenantMasterExtension;
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use k1s0_auth::actor_from_claims;
use k1s0_auth::Claims;
use k1s0_server_common::ServiceError;
use uuid::Uuid;

pub async fn get_tenant_extension(
    State(state): State<AppState>,
    Path((tenant_id, item_id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse, ServiceError> {
    let extension = state
        .manage_tenant_extensions_uc
        .get_extension(&tenant_id, item_id)
        .await
        .map_err(from_anyhow)?
        .ok_or_else(|| ServiceError::NotFound {
            code: k1s0_server_common::ErrorCode::new("BIZ_DOMAINMASTER_ITEM_NOT_FOUND"),
            message: format!(
                "Tenant extension not found for tenant '{}', item '{}'",
                tenant_id, item_id
            ),
        })?;
    Ok(Json(extension))
}

pub async fn upsert_tenant_extension(
    State(state): State<AppState>,
    Path((tenant_id, item_id)): Path<(String, Uuid)>,
    claims: Option<Extension<Claims>>,
    Json(input): Json<UpsertTenantMasterExtension>,
) -> Result<impl IntoResponse, ServiceError> {
    let actor = actor_from_claims(claims.as_ref().map(|Extension(claims)| claims));
    let extension = state
        .manage_tenant_extensions_uc
        .upsert_extension(&tenant_id, item_id, &input, &actor)
        .await
        .map_err(from_anyhow)?;
    Ok(Json(extension))
}

pub async fn delete_tenant_extension(
    State(state): State<AppState>,
    Path((tenant_id, item_id)): Path<(String, Uuid)>,
    claims: Option<Extension<Claims>>,
) -> Result<StatusCode, ServiceError> {
    let actor = actor_from_claims(claims.as_ref().map(|Extension(claims)| claims));
    state
        .manage_tenant_extensions_uc
        .delete_extension(&tenant_id, item_id, &actor)
        .await
        .map_err(from_anyhow)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_tenant_items(
    State(state): State<AppState>,
    Path((tenant_id, category_code)): Path<(String, String)>,
) -> Result<impl IntoResponse, ServiceError> {
    let merged_items = state
        .manage_tenant_extensions_uc
        .list_tenant_items(&tenant_id, &category_code)
        .await
        .map_err(from_anyhow)?;
    Ok(Json(serde_json::json!({ "items": merged_items })))
}
