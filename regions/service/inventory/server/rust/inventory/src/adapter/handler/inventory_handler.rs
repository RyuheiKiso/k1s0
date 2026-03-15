use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
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

pub async fn reserve_stock(
    State(state): State<AppState>,
    Json(body): Json<ReserveStockRequest>,
) -> Result<impl IntoResponse, ServiceError> {
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

pub async fn release_stock(
    State(state): State<AppState>,
    Json(body): Json<ReleaseStockRequest>,
) -> Result<impl IntoResponse, ServiceError> {
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

pub async fn get_inventory(
    State(state): State<AppState>,
    Path(inventory_id): Path<String>,
) -> Result<impl IntoResponse, ServiceError> {
    let id = parse_uuid(&inventory_id)?;

    let item = state
        .get_inventory_uc
        .execute(id)
        .await
        .map_err(map_inventory_error)?;

    let response = InventoryDetailResponse::from_entity(&item);
    Ok(Json(response))
}

pub async fn list_inventory(
    State(state): State<AppState>,
    Query(query): Query<ListInventoryQuery>,
) -> Result<impl IntoResponse, ServiceError> {
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

pub async fn update_stock(
    State(state): State<AppState>,
    Path(inventory_id): Path<String>,
    Json(body): Json<UpdateStockRequest>,
) -> Result<impl IntoResponse, ServiceError> {
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
