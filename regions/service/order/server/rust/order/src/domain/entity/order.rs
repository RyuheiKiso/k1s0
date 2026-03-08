use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 注文ステータス。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
}

impl OrderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Confirmed => "confirmed",
            Self::Processing => "processing",
            Self::Shipped => "shipped",
            Self::Delivered => "delivered",
            Self::Cancelled => "cancelled",
        }
    }

    /// ステータス遷移が有効かどうかを検証する。
    pub fn can_transition_to(&self, next: &Self) -> bool {
        matches!(
            (self, next),
            (Self::Pending, Self::Confirmed)
                | (Self::Pending, Self::Cancelled)
                | (Self::Confirmed, Self::Processing)
                | (Self::Confirmed, Self::Cancelled)
                | (Self::Processing, Self::Shipped)
                | (Self::Processing, Self::Cancelled)
                | (Self::Shipped, Self::Delivered)
        )
    }
}

impl std::str::FromStr for OrderStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "confirmed" => Ok(Self::Confirmed),
            "processing" => Ok(Self::Processing),
            "shipped" => Ok(Self::Shipped),
            "delivered" => Ok(Self::Delivered),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(format!("invalid order status: '{}'", s)),
        }
    }
}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 注文エンティティ。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub customer_id: String,
    pub status: OrderStatus,
    pub total_amount: i64,
    pub currency: String,
    pub notes: Option<String>,
    pub created_by: String,
    pub updated_by: Option<String>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 注文明細エンティティ。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub product_id: String,
    pub product_name: String,
    pub quantity: i32,
    pub unit_price: i64,
    pub subtotal: i64,
    pub created_at: DateTime<Utc>,
}

/// 注文作成リクエスト。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrder {
    pub customer_id: String,
    pub currency: String,
    pub notes: Option<String>,
    pub items: Vec<CreateOrderItem>,
}

/// 注文明細作成リクエスト。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderItem {
    pub product_id: String,
    pub product_name: String,
    pub quantity: i32,
    pub unit_price: i64,
}

/// 注文一覧フィルター。
#[derive(Debug, Clone, Default)]
pub struct OrderFilter {
    pub customer_id: Option<String>,
    pub status: Option<OrderStatus>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_status_roundtrip() {
        let statuses = vec![
            OrderStatus::Pending,
            OrderStatus::Confirmed,
            OrderStatus::Processing,
            OrderStatus::Shipped,
            OrderStatus::Delivered,
            OrderStatus::Cancelled,
        ];
        for status in statuses {
            let s = status.as_str();
            let parsed: OrderStatus = s.parse().unwrap();
            assert_eq!(parsed, status);
        }
    }

    #[test]
    fn test_order_status_invalid() {
        let result = "unknown".parse::<OrderStatus>();
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_transitions() {
        assert!(OrderStatus::Pending.can_transition_to(&OrderStatus::Confirmed));
        assert!(OrderStatus::Pending.can_transition_to(&OrderStatus::Cancelled));
        assert!(OrderStatus::Confirmed.can_transition_to(&OrderStatus::Processing));
        assert!(OrderStatus::Processing.can_transition_to(&OrderStatus::Shipped));
        assert!(OrderStatus::Shipped.can_transition_to(&OrderStatus::Delivered));
    }

    #[test]
    fn test_invalid_transitions() {
        assert!(!OrderStatus::Pending.can_transition_to(&OrderStatus::Shipped));
        assert!(!OrderStatus::Delivered.can_transition_to(&OrderStatus::Pending));
        assert!(!OrderStatus::Cancelled.can_transition_to(&OrderStatus::Confirmed));
    }

    #[test]
    fn test_create_order_item_subtotal() {
        let item = CreateOrderItem {
            product_id: "PROD-001".to_string(),
            product_name: "Widget".to_string(),
            quantity: 3,
            unit_price: 1000,
        };
        assert_eq!(item.quantity as i64 * item.unit_price, 3000);
    }
}
