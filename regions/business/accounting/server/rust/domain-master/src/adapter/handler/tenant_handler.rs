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

/// テナント拡張単件取得ハンドラー。read 操作も認証必須（P0-2 対応）。
pub async fn get_tenant_extension(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path((tenant_id, item_id)): Path<(String, Uuid)>,
) -> Result<impl IntoResponse, ServiceError> {
    // read 操作も認証が必要（gRPC ハンドラーと同等の認証強度を維持する）
    let _ = claims
        .ok_or_else(|| ServiceError::unauthorized("BIZ_DOMAINMASTER", "authentication required"))?;
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

/// テナント拡張アップサートハンドラー。Claims が存在しない場合は 401 Unauthorized を返す。
pub async fn upsert_tenant_extension(
    State(state): State<AppState>,
    Path((tenant_id, item_id)): Path<(String, Uuid)>,
    claims: Option<Extension<Claims>>,
    Json(input): Json<UpsertTenantMasterExtension>,
) -> Result<impl IntoResponse, ServiceError> {
    // Claims が存在しない（未認証）場合は 401 を返す。actor_from_claims は None 時に
    // "anonymous" を返すため、明示的な認証チェックが必要（P0-2 対応）。
    let claims = claims
        .ok_or_else(|| ServiceError::unauthorized("BIZ_DOMAINMASTER", "authentication required"))?;
    let actor = actor_from_claims(Some(&claims.0));
    let extension = state
        .manage_tenant_extensions_uc
        .upsert_extension(&tenant_id, item_id, &input, &actor)
        .await
        .map_err(from_anyhow)?;
    Ok(Json(extension))
}

/// テナント拡張削除ハンドラー。Claims が存在しない場合は 401 Unauthorized を返す。
pub async fn delete_tenant_extension(
    State(state): State<AppState>,
    Path((tenant_id, item_id)): Path<(String, Uuid)>,
    claims: Option<Extension<Claims>>,
) -> Result<StatusCode, ServiceError> {
    // Claims が存在しない（未認証）場合は 401 を返す。actor_from_claims は None 時に
    // "anonymous" を返すため、明示的な認証チェックが必要（P0-2 対応）。
    let claims = claims
        .ok_or_else(|| ServiceError::unauthorized("BIZ_DOMAINMASTER", "authentication required"))?;
    let actor = actor_from_claims(Some(&claims.0));
    state
        .manage_tenant_extensions_uc
        .delete_extension(&tenant_id, item_id, &actor)
        .await
        .map_err(from_anyhow)?;
    Ok(StatusCode::NO_CONTENT)
}

/// テナントアイテム一覧取得ハンドラー。read 操作も認証必須（P0-2 対応）。
pub async fn list_tenant_items(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path((tenant_id, category_code)): Path<(String, String)>,
) -> Result<impl IntoResponse, ServiceError> {
    // read 操作も認証が必要（gRPC ハンドラーと同等の認証強度を維持する）
    let _ = claims
        .ok_or_else(|| ServiceError::unauthorized("BIZ_DOMAINMASTER", "authentication required"))?;
    let merged_items = state
        .manage_tenant_extensions_uc
        .list_tenant_items(&tenant_id, &category_code)
        .await
        .map_err(from_anyhow)?;
    Ok(Json(serde_json::json!({ "items": merged_items })))
}
