//! Presenter レイヤー — ドメインエンティティを API レスポンス形式に変換する。

use crate::domain::entity::order::{Order, OrderItem};
use serde::Serialize;

/// 注文詳細 API レスポンス。
#[derive(Debug, Serialize)]
pub struct OrderDetailResponse {
    pub id: String,
    pub customer_id: String,
    pub status: String,
    pub total_amount: i64,
    pub currency: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    pub items: Vec<OrderItemResponse>,
    pub created_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_by: Option<String>,
    pub version: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// 注文明細 API レスポンス。
#[derive(Debug, Serialize)]
pub struct OrderItemResponse {
    pub id: String,
    pub product_id: String,
    pub product_name: String,
    pub quantity: i32,
    pub unit_price: i64,
    pub subtotal: i64,
}

/// 注文一覧 API レスポンス。
#[derive(Debug, Serialize)]
pub struct OrderListResponse {
    pub orders: Vec<OrderSummaryResponse>,
    pub total: i64,
}

/// 注文サマリ（一覧表示用）。
#[derive(Debug, Serialize)]
pub struct OrderSummaryResponse {
    pub id: String,
    pub customer_id: String,
    pub status: String,
    pub total_amount: i64,
    pub currency: String,
    pub created_at: String,
    pub updated_at: String,
}

impl OrderDetailResponse {
    pub fn from_entities(order: &Order, items: &[OrderItem]) -> Self {
        Self {
            id: order.id.to_string(),
            customer_id: order.customer_id.clone(),
            status: order.status.as_str().to_string(),
            total_amount: order.total_amount,
            currency: order.currency.clone(),
            notes: order.notes.clone(),
            items: items.iter().map(OrderItemResponse::from).collect(),
            created_by: order.created_by.clone(),
            updated_by: order.updated_by.clone(),
            version: order.version,
            created_at: order.created_at.to_rfc3339(),
            updated_at: order.updated_at.to_rfc3339(),
        }
    }
}

impl From<&OrderItem> for OrderItemResponse {
    fn from(item: &OrderItem) -> Self {
        Self {
            id: item.id.to_string(),
            product_id: item.product_id.clone(),
            product_name: item.product_name.clone(),
            quantity: item.quantity,
            unit_price: item.unit_price,
            subtotal: item.subtotal,
        }
    }
}

impl OrderListResponse {
    pub fn from_entities(orders: &[Order], total: i64) -> Self {
        Self {
            orders: orders.iter().map(OrderSummaryResponse::from).collect(),
            total,
        }
    }
}

impl From<&Order> for OrderSummaryResponse {
    fn from(order: &Order) -> Self {
        Self {
            id: order.id.to_string(),
            customer_id: order.customer_id.clone(),
            status: order.status.as_str().to_string(),
            total_amount: order.total_amount,
            currency: order.currency.clone(),
            created_at: order.created_at.to_rfc3339(),
            updated_at: order.updated_at.to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::order::OrderStatus;
    use chrono::Utc;
    use uuid::Uuid;

    fn sample_order() -> Order {
        Order {
            id: Uuid::new_v4(),
            customer_id: "CUST-001".to_string(),
            status: OrderStatus::Pending,
            total_amount: 3000,
            currency: "JPY".to_string(),
            notes: Some("rush delivery".to_string()),
            created_by: "admin".to_string(),
            updated_by: None,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn sample_items(order_id: Uuid) -> Vec<OrderItem> {
        vec![OrderItem {
            id: Uuid::new_v4(),
            order_id,
            product_id: "PROD-001".to_string(),
            product_name: "Widget".to_string(),
            quantity: 3,
            unit_price: 1000,
            subtotal: 3000,
            created_at: Utc::now(),
        }]
    }

    #[test]
    fn test_order_detail_response() {
        let order = sample_order();
        let items = sample_items(order.id);
        let resp = OrderDetailResponse::from_entities(&order, &items);
        assert_eq!(resp.customer_id, "CUST-001");
        assert_eq!(resp.status, "pending");
        assert_eq!(resp.items.len(), 1);
        assert_eq!(resp.items[0].subtotal, 3000);
    }

    #[test]
    fn test_order_list_response() {
        let orders = vec![sample_order()];
        let resp = OrderListResponse::from_entities(&orders, 1);
        assert_eq!(resp.total, 1);
        assert_eq!(resp.orders.len(), 1);
        assert_eq!(resp.orders[0].status, "pending");
    }

    #[test]
    fn test_order_summary_response() {
        let order = sample_order();
        let resp = OrderSummaryResponse::from(&order);
        assert_eq!(resp.customer_id, "CUST-001");
        assert_eq!(resp.total_amount, 3000);
    }
}
