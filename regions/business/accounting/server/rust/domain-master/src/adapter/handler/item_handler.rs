use crate::adapter::handler::error::from_anyhow;
use crate::adapter::handler::AppState;
use crate::domain::entity::master_item::{CreateMasterItem, UpdateMasterItem};
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use k1s0_auth::actor_from_claims;
use k1s0_auth::Claims;
use k1s0_server_common::ServiceError;
use serde::Deserialize;

/// 一覧取得用クエリパラメータ。
#[derive(Debug, Deserialize)]
pub struct ListItemsQuery {
    pub active_only: Option<bool>,
}

/// アイテム一覧取得ハンドラー。read 操作も認証必須（P0-2 対応）。
pub async fn list_items(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(category_code): Path<String>,
    Query(query): Query<ListItemsQuery>,
) -> Result<impl IntoResponse, ServiceError> {
    // read 操作も認証が必要（gRPC ハンドラーと同等の認証強度を維持する）
    let _guard = claims
        .ok_or_else(|| ServiceError::unauthorized("BIZ_DOMAINMASTER", "authentication required"))?;
    let items = state
        .manage_items_uc
        .list_items(&category_code, query.active_only.unwrap_or(false))
        .await
        .map_err(from_anyhow)?;
    Ok(Json(serde_json::json!({ "items": items })))
}

/// アイテム単件取得ハンドラー。read 操作も認証必須（P0-2 対応）。
pub async fn get_item(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path((category_code, item_code)): Path<(String, String)>,
) -> Result<impl IntoResponse, ServiceError> {
    // read 操作も認証が必要（gRPC ハンドラーと同等の認証強度を維持する）
    let _guard = claims
        .ok_or_else(|| ServiceError::unauthorized("BIZ_DOMAINMASTER", "authentication required"))?;
    let item = state
        .manage_items_uc
        .get_item(&category_code, &item_code)
        .await
        .map_err(from_anyhow)?
        .ok_or_else(|| ServiceError::NotFound {
            code: k1s0_server_common::ErrorCode::new("BIZ_DOMAINMASTER_ITEM_NOT_FOUND"),
            message: format!(
                "Item '{}' not found in category '{}'",
                item_code, category_code
            ),
        })?;
    Ok(Json(item))
}

/// アイテム作成ハンドラー。Claims が存在しない場合は 401 Unauthorized を返す。
pub async fn create_item(
    State(state): State<AppState>,
    Path(category_code): Path<String>,
    claims: Option<Extension<Claims>>,
    Json(input): Json<CreateMasterItem>,
) -> Result<impl IntoResponse, ServiceError> {
    // Claims が存在しない（未認証）場合は 401 を返す。actor_from_claims は None 時に
    // "anonymous" を返すため、明示的な認証チェックが必要（P0-2 対応）。
    let claims = claims
        .ok_or_else(|| ServiceError::unauthorized("BIZ_DOMAINMASTER", "authentication required"))?;
    let actor = actor_from_claims(Some(&claims.0));
    let item = state
        .manage_items_uc
        .create_item(&category_code, &input, &actor)
        .await
        .map_err(from_anyhow)?;
    Ok((StatusCode::CREATED, Json(item)))
}

/// アイテム更新ハンドラー。Claims が存在しない場合は 401 Unauthorized を返す。
pub async fn update_item(
    State(state): State<AppState>,
    Path((category_code, item_code)): Path<(String, String)>,
    claims: Option<Extension<Claims>>,
    Json(input): Json<UpdateMasterItem>,
) -> Result<impl IntoResponse, ServiceError> {
    // Claims が存在しない（未認証）場合は 401 を返す。actor_from_claims は None 時に
    // "anonymous" を返すため、明示的な認証チェックが必要（P0-2 対応）。
    let claims = claims
        .ok_or_else(|| ServiceError::unauthorized("BIZ_DOMAINMASTER", "authentication required"))?;
    let actor = actor_from_claims(Some(&claims.0));
    let item = state
        .manage_items_uc
        .update_item(&category_code, &item_code, &input, &actor)
        .await
        .map_err(from_anyhow)?;
    Ok(Json(item))
}

/// アイテム削除ハンドラー。Claims が存在しない場合は 401 Unauthorized を返す。
pub async fn delete_item(
    State(state): State<AppState>,
    Path((category_code, item_code)): Path<(String, String)>,
    claims: Option<Extension<Claims>>,
) -> Result<StatusCode, ServiceError> {
    // Claims が存在しない（未認証）場合は 401 を返す。actor_from_claims は None 時に
    // "anonymous" を返すため、明示的な認証チェックが必要（P0-2 対応）。
    let claims = claims
        .ok_or_else(|| ServiceError::unauthorized("BIZ_DOMAINMASTER", "authentication required"))?;
    let actor = actor_from_claims(Some(&claims.0));
    state
        .manage_items_uc
        .delete_item(&category_code, &item_code, &actor)
        .await
        .map_err(from_anyhow)?;
    Ok(StatusCode::NO_CONTENT)
}
