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
use crate::adapter::presenter::order_presenter::{OrderDetailResponse, OrderListResponse};
use crate::domain::entity::order::{CreateOrder, OrderFilter, OrderStatus};
use crate::domain::error::OrderError;

/// 注文作成リクエストボディ。
#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    pub customer_id: String,
    pub currency: String,
    pub notes: Option<String>,
    pub items: Vec<CreateOrderItemRequest>,
}

#[derive(Debug, Deserialize)]
pub struct CreateOrderItemRequest {
    pub product_id: String,
    pub product_name: String,
    pub quantity: i32,
    pub unit_price: i64,
}

/// ステータス更新リクエストボディ。
#[derive(Debug, Deserialize)]
pub struct UpdateOrderStatusRequest {
    pub status: String,
}

/// 一覧取得用クエリパラメータ。
#[derive(Debug, Deserialize)]
pub struct ListOrdersQuery {
    pub customer_id: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// 注文作成ハンドラー。Claims が存在しない場合は 401 Unauthorized を返す。
pub async fn create_order(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Json(body): Json<CreateOrderRequest>,
) -> Result<impl IntoResponse, ServiceError> {
    // Claims が存在しない（未認証）場合は 401 を返す。actor_from_claims は None 時に
    // "anonymous" を返すため、明示的な認証チェックが必要（P0-2 対応）。
    let claims = claims
        .ok_or_else(|| ServiceError::unauthorized("ORDER", "authentication required"))?;
    let actor = actor_from_claims(Some(&claims.0));
    tracing::info!(actor = %actor, "create_order invoked");

    let input = CreateOrder {
        customer_id: body.customer_id,
        currency: body.currency,
        notes: body.notes,
        items: body
            .items
            .into_iter()
            .map(|item| crate::domain::entity::order::CreateOrderItem {
                product_id: item.product_id,
                product_name: item.product_name,
                quantity: item.quantity,
                unit_price: item.unit_price,
            })
            .collect(),
    };

    let (order, items) = state
        .create_order_uc
        .execute(&input, &actor)
        .await
        .map_err(map_order_error)?;

    let response = OrderDetailResponse::from_entities(&order, &items);
    Ok((StatusCode::CREATED, Json(response)))
}

/// 注文取得ハンドラー。read 操作も認証必須（P0-2 対応）。
pub async fn get_order(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(order_id): Path<String>,
) -> Result<impl IntoResponse, ServiceError> {
    // read 操作も認証が必要（gRPC ハンドラーと同等の認証強度を維持する）
    let _guard = claims.ok_or_else(|| ServiceError::unauthorized("ORDER", "authentication required"))?;
    let id = parse_uuid(&order_id)?;

    let (order, items) = state
        .get_order_uc
        .execute(id)
        .await
        .map_err(map_order_error)?;

    let response = OrderDetailResponse::from_entities(&order, &items);
    Ok(Json(response))
}

/// 注文ステータス更新ハンドラー。Claims が存在しない場合は 401 Unauthorized を返す。
pub async fn update_order_status(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(order_id): Path<String>,
    Json(body): Json<UpdateOrderStatusRequest>,
) -> Result<impl IntoResponse, ServiceError> {
    // Claims が存在しない（未認証）場合は 401 を返す（P0-2 対応）。
    let claims = claims
        .ok_or_else(|| ServiceError::unauthorized("ORDER", "authentication required"))?;
    let actor = actor_from_claims(Some(&claims.0));
    tracing::info!(actor = %actor, "update_order_status invoked");
    let id = parse_uuid(&order_id)?;

    let new_status = body
        .status
        .parse::<OrderStatus>()
        .map_err(|_| ServiceError::BadRequest {
            code: k1s0_server_common::error::ErrorCode::new("SVC_ORDER_VALIDATION_FAILED"),
            message: format!("invalid order status: '{}'", body.status),
            details: vec![],
        })?;

    let order = state
        .update_order_status_uc
        .execute(id, &new_status, &actor)
        .await
        .map_err(map_order_error)?;

    let items = state
        .get_order_uc
        .execute(order.id)
        .await
        .map(|(_, items)| items)
        .map_err(|err| {
            tracing::error!(error = %err, order_id = %order.id, "failed to fetch items after status update");
            ServiceError::Internal {
                code: k1s0_server_common::error::ErrorCode::new("SVC_ORDER_INTERNAL_ERROR"),
                message: "failed to fetch order items".to_string(),
            }
        })?;

    let response = OrderDetailResponse::from_entities(&order, &items);
    Ok(Json(response))
}

/// 注文一覧ハンドラー。read 操作も認証必須（P0-2 対応）。
pub async fn list_orders(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Query(query): Query<ListOrdersQuery>,
) -> Result<impl IntoResponse, ServiceError> {
    // read 操作も認証が必要（gRPC ハンドラーと同等の認証強度を維持する）
    let _guard = claims.ok_or_else(|| ServiceError::unauthorized("ORDER", "authentication required"))?;
    let status = match &query.status {
        Some(s) => Some(
            s.parse::<OrderStatus>()
                .map_err(|_| ServiceError::BadRequest {
                    code: k1s0_server_common::error::ErrorCode::new("SVC_ORDER_VALIDATION_FAILED"),
                    message: format!("invalid order status: '{}'", s),
                    details: vec![],
                })?,
        ),
        None => None,
    };

    let filter = OrderFilter {
        customer_id: query.customer_id,
        status,
        limit: query.limit.or(Some(50)),
        offset: query.offset.or(Some(0)),
    };

    let (orders, total) = state
        .list_orders_uc
        .execute(&filter)
        .await
        .map_err(map_order_error)?;

    let response = OrderListResponse::from_entities(&orders, total);
    Ok(Json(response))
}

/// UUID パースヘルパー。
fn parse_uuid(s: &str) -> Result<Uuid, ServiceError> {
    Uuid::parse_str(s).map_err(|_| ServiceError::BadRequest {
        code: k1s0_server_common::error::ErrorCode::new("SVC_ORDER_VALIDATION_FAILED"),
        message: format!("invalid order_id format: '{}'", s),
        details: vec![k1s0_server_common::error::ErrorDetail::new(
            "order_id",
            "invalid_format",
            "must be a valid UUID",
        )],
    })
}

/// anyhow::Error を ServiceError に変換する。
///
/// OrderError がダウンキャスト可能な場合は型安全に変換し、
/// それ以外は Internal エラーとして扱う。
fn map_order_error(err: anyhow::Error) -> ServiceError {
    match err.downcast::<OrderError>() {
        Ok(order_err) => order_err.into(),
        Err(other) => ServiceError::Internal {
            code: k1s0_server_common::error::ErrorCode::new("SVC_ORDER_INTERNAL_ERROR"),
            message: other.to_string(),
        },
    }
}
