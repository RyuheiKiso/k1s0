use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Extension;
use axum::Json;
use k1s0_auth::{actor_from_claims, Claims};
use k1s0_server_common::error::ServiceError;
use serde::Deserialize;
use uuid::Uuid;

use super::AppState;
use crate::adapter::presenter::inventory_presenter::{
    InventoryDetailResponse, InventoryListResponse,
};
use crate::domain::entity::inventory_item::InventoryFilter;
use crate::domain::error::InventoryError;

/// 在庫予約リクエストボディ。
#[derive(Debug, Deserialize)]
pub struct ReserveStockRequest {
    pub order_id: String,
    pub product_id: String,
    pub warehouse_id: String,
    pub quantity: i32,
}

/// 在庫解放リクエストボディ。
#[derive(Debug, Deserialize)]
pub struct ReleaseStockRequest {
    pub order_id: String,
    pub product_id: String,
    pub warehouse_id: String,
    pub quantity: i32,
    pub reason: String,
}

/// 在庫更新リクエストボディ。
#[derive(Debug, Deserialize)]
pub struct UpdateStockRequest {
    pub qty_available: i32,
    pub expected_version: i32,
}

/// 一覧取得用クエリパラメータ。
#[derive(Debug, Deserialize)]
pub struct ListInventoryQuery {
    pub product_id: Option<String>,
    pub warehouse_id: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// 在庫予約ハンドラー。Claims が存在しない場合は 401 Unauthorized を返す。
pub async fn reserve_stock(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Json(body): Json<ReserveStockRequest>,
) -> Result<impl IntoResponse, ServiceError> {
    // Claims が存在しない（未認証）場合は 401 を返す。actor_from_claims は None 時に
    // "anonymous" を返すため、明示的な認証チェックが必要（P0-2 対応）。
    let claims = claims
        .ok_or_else(|| ServiceError::unauthorized("INVENTORY", "authentication required"))?;
    let actor = actor_from_claims(Some(&claims.0));
    tracing::info!(actor = %actor, "reserve_stock invoked");

    let item = state
        .reserve_stock_uc
        .execute(
            &body.product_id,
            &body.warehouse_id,
            body.quantity,
            &body.order_id,
        )
        .await
        .map_err(map_inventory_error)?;

    let response = InventoryDetailResponse::from_entity(&item);
    Ok((StatusCode::OK, Json(response)))
}

/// 在庫解放ハンドラー。Claims が存在しない場合は 401 Unauthorized を返す。
pub async fn release_stock(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Json(body): Json<ReleaseStockRequest>,
) -> Result<impl IntoResponse, ServiceError> {
    // Claims が存在しない（未認証）場合は 401 を返す（P0-2 対応）。
    let claims = claims
        .ok_or_else(|| ServiceError::unauthorized("INVENTORY", "authentication required"))?;
    let actor = actor_from_claims(Some(&claims.0));
    tracing::info!(actor = %actor, "release_stock invoked");

    let item = state
        .release_stock_uc
        .execute(
            &body.product_id,
            &body.warehouse_id,
            body.quantity,
            &body.order_id,
            &body.reason,
        )
        .await
        .map_err(map_inventory_error)?;

    let response = InventoryDetailResponse::from_entity(&item);
    Ok((StatusCode::OK, Json(response)))
}

/// 在庫取得ハンドラー。read 操作も認証必須（P0-2 対応）。
pub async fn get_inventory(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(inventory_id): Path<String>,
) -> Result<impl IntoResponse, ServiceError> {
    // read 操作も認証が必要（gRPC ハンドラーと同等の認証強度を維持する）
    claims.ok_or_else(|| ServiceError::unauthorized("INVENTORY", "authentication required"))?;
    let id = parse_uuid(&inventory_id)?;

    let item = state
        .get_inventory_uc
        .execute(id)
        .await
        .map_err(map_inventory_error)?;

    let response = InventoryDetailResponse::from_entity(&item);
    Ok(Json(response))
}

/// 在庫一覧ハンドラー。read 操作も認証必須（P0-2 対応）。
pub async fn list_inventory(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Query(query): Query<ListInventoryQuery>,
) -> Result<impl IntoResponse, ServiceError> {
    // read 操作も認証が必要（gRPC ハンドラーと同等の認証強度を維持する）
    claims.ok_or_else(|| ServiceError::unauthorized("INVENTORY", "authentication required"))?;
    let filter = InventoryFilter {
        product_id: query.product_id,
        warehouse_id: query.warehouse_id,
        limit: query.limit.or(Some(50)),
        offset: query.offset.or(Some(0)),
    };

    let (items, total) = state
        .list_inventory_uc
        .execute(&filter)
        .await
        .map_err(map_inventory_error)?;

    let response = InventoryListResponse::from_entities(&items, total);
    Ok(Json(response))
}

/// 在庫更新ハンドラー。Claims が存在しない場合は 401 Unauthorized を返す。
pub async fn update_stock(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(inventory_id): Path<String>,
    Json(body): Json<UpdateStockRequest>,
) -> Result<impl IntoResponse, ServiceError> {
    // Claims が存在しない（未認証）場合は 401 を返す（P0-2 対応）。
    let claims = claims
        .ok_or_else(|| ServiceError::unauthorized("INVENTORY", "authentication required"))?;
    let actor = actor_from_claims(Some(&claims.0));
    tracing::info!(actor = %actor, "update_stock invoked");
    let id = parse_uuid(&inventory_id)?;

    let item = state
        .update_stock_uc
        .execute(id, body.qty_available, body.expected_version)
        .await
        .map_err(map_inventory_error)?;

    let response = InventoryDetailResponse::from_entity(&item);
    Ok(Json(response))
}

/// UUID パースヘルパー。
fn parse_uuid(s: &str) -> Result<Uuid, ServiceError> {
    Uuid::parse_str(s).map_err(|_| ServiceError::BadRequest {
        code: k1s0_server_common::error::ErrorCode::new("SVC_INVENTORY_VALIDATION_FAILED"),
        message: format!("invalid inventory_id format: '{}'", s),
        details: vec![k1s0_server_common::error::ErrorDetail::new(
            "inventory_id",
            "invalid_format",
            "must be a valid UUID",
        )],
    })
}

/// anyhow::Error を ServiceError に変換する。
///
/// InventoryError がダウンキャスト可能な場合は型安全に変換し、
/// それ以外は Internal エラーとして扱う。
fn map_inventory_error(err: anyhow::Error) -> ServiceError {
    match err.downcast::<InventoryError>() {
        Ok(inventory_err) => inventory_err.into(),
        Err(other) => ServiceError::Internal {
            code: k1s0_server_common::error::ErrorCode::new("SVC_INVENTORY_INTERNAL_ERROR"),
            message: other.to_string(),
        },
    }
}
