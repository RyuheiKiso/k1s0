use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Extension;
use axum::Json;
use k1s0_auth::Claims;
use serde::Deserialize;
use uuid::Uuid;

use super::AppState;
use crate::adapter::presenter::order_presenter::{OrderDetailResponse, OrderListResponse};
use crate::domain::entity::order::{CreateOrder, OrderFilter, OrderStatus};

use super::actor_from_claims;

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

pub async fn create_order(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Json(body): Json<CreateOrderRequest>,
) -> impl IntoResponse {
    let actor = actor_from_claims(claims.as_ref().map(|c| &c.0));

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

    match state.create_order_uc.execute(&input, &actor).await {
        Ok((order, items)) => {
            let response = OrderDetailResponse::from_entities(&order, &items);
            (StatusCode::CREATED, Json(serde_json::to_value(response).unwrap())).into_response()
        }
        Err(err) => from_anyhow(err).into_response(),
    }
}

pub async fn get_order(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
) -> impl IntoResponse {
    let id = match Uuid::parse_str(&order_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "invalid order_id format"})),
            )
                .into_response();
        }
    };

    match state.get_order_uc.execute(id).await {
        Ok((order, items)) => {
            let response = OrderDetailResponse::from_entities(&order, &items);
            (StatusCode::OK, Json(serde_json::to_value(response).unwrap())).into_response()
        }
        Err(err) => from_anyhow(err).into_response(),
    }
}

pub async fn update_order_status(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(order_id): Path<String>,
    Json(body): Json<UpdateOrderStatusRequest>,
) -> impl IntoResponse {
    let actor = actor_from_claims(claims.as_ref().map(|c| &c.0));

    let id = match Uuid::parse_str(&order_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "invalid order_id format"})),
            )
                .into_response();
        }
    };

    let new_status = match OrderStatus::from_str(&body.status) {
        Ok(s) => s,
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": err.to_string()})),
            )
                .into_response();
        }
    };

    match state
        .update_order_status_uc
        .execute(id, &new_status, &actor)
        .await
    {
        Ok(order) => {
            let items = state
                .get_order_uc
                .execute(order.id)
                .await
                .map(|(_, items)| items)
                .unwrap_or_default();
            let response = OrderDetailResponse::from_entities(&order, &items);
            (StatusCode::OK, Json(serde_json::to_value(response).unwrap())).into_response()
        }
        Err(err) => from_anyhow(err).into_response(),
    }
}

pub async fn list_orders(
    State(state): State<AppState>,
    Query(query): Query<ListOrdersQuery>,
) -> impl IntoResponse {
    let status = match &query.status {
        Some(s) => match OrderStatus::from_str(s) {
            Ok(status) => Some(status),
            Err(err) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({"error": err.to_string()})),
                )
                    .into_response();
            }
        },
        None => None,
    };

    let filter = OrderFilter {
        customer_id: query.customer_id,
        status,
        limit: query.limit.or(Some(50)),
        offset: query.offset.or(Some(0)),
    };

    match state.list_orders_uc.execute(&filter).await {
        Ok((orders, total)) => {
            let response = OrderListResponse::from_entities(&orders, total);
            (StatusCode::OK, Json(serde_json::to_value(response).unwrap())).into_response()
        }
        Err(err) => from_anyhow(err).into_response(),
    }
}

/// anyhow::Error を HTTP レスポンスに変換するヘルパー。
fn from_anyhow(err: anyhow::Error) -> impl IntoResponse {
    let msg = err.to_string();
    let lower = msg.to_ascii_lowercase();

    if lower.contains("not found") {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "code": "SVC_ORDER_NOT_FOUND",
                "message": msg,
            })),
        );
    }
    if lower.contains("invalid status transition") {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "code": "SVC_ORDER_INVALID_STATUS_TRANSITION",
                "message": msg,
            })),
        );
    }
    if lower.contains("must not be empty")
        || lower.contains("must be greater")
        || lower.contains("must not be negative")
        || lower.contains("at least one item")
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "code": "SVC_ORDER_VALIDATION_ERROR",
                "message": msg,
            })),
        );
    }

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
            "code": "SVC_ORDER_INTERNAL_ERROR",
            "message": msg,
        })),
    )
}
