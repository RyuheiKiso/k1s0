use crate::domain::error::OrderError;
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

impl Order {
    /// ステータス遷移を検証し、遷移後のステータスを返す。
    /// ドメインルールをエンティティに集約し、貧血ドメインモデルを防ぐ（H-005）。
    /// 無効な遷移は OrderError::InvalidStatusTransition を返す。
    pub fn transition_to(&self, next: OrderStatus) -> Result<OrderStatus, OrderError> {
        if !self.status.can_transition_to(&next) {
            return Err(OrderError::InvalidStatusTransition {
                from: self.status.to_string(),
                to: next.to_string(),
            });
        }
        Ok(next)
    }
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

impl CreateOrder {
    /// 入力値のドメインバリデーションを実行する。
    /// バリデーションロジックをエンティティに集約し、ドメインルールを一箇所に保つ（M-001）。
    pub fn validate(&self) -> Result<(), OrderError> {
        if self.customer_id.trim().is_empty() {
            return Err(OrderError::ValidationFailed(
                "customer_id must not be empty".to_string(),
            ));
        }
        if self.currency.trim().is_empty() {
            return Err(OrderError::ValidationFailed(
                "currency must not be empty".to_string(),
            ));
        }
        if self.items.is_empty() {
            return Err(OrderError::ValidationFailed(
                "order must contain at least one item".to_string(),
            ));
        }
        for (i, item) in self.items.iter().enumerate() {
            if item.product_id.trim().is_empty() {
                return Err(OrderError::ValidationFailed(format!(
                    "items[{}].product_id must not be empty",
                    i
                )));
            }
            if item.product_name.trim().is_empty() {
                return Err(OrderError::ValidationFailed(format!(
                    "items[{}].product_name must not be empty",
                    i
                )));
            }
            if item.quantity <= 0 {
                return Err(OrderError::ValidationFailed(format!(
                    "items[{}].quantity must be greater than zero",
                    i
                )));
            }
            if item.unit_price < 0 {
                return Err(OrderError::ValidationFailed(format!(
                    "items[{}].unit_price must not be negative",
                    i
                )));
            }
        }
        Ok(())
    }

    /// 明細から合計金額を計算する。
    /// 計算ロジックをエンティティに集約し、usecase 側の重複を排除する（M-001）。
    pub fn calculate_total(&self) -> i64 {
        self.items
            .iter()
            .map(|item| item.quantity as i64 * item.unit_price)
            .sum()
    }
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

    #[test]
    fn test_order_transition_to_valid() {
        // Order::transition_to() が有効な遷移で次ステータスを返すことを確認する
        let order = Order {
            id: uuid::Uuid::new_v4(),
            customer_id: "CUST-001".to_string(),
            status: OrderStatus::Pending,
            total_amount: 1000,
            currency: "JPY".to_string(),
            notes: None,
            created_by: "admin".to_string(),
            updated_by: None,
            version: 1,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let result = order.transition_to(OrderStatus::Confirmed);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), OrderStatus::Confirmed);
    }

    #[test]
    fn test_order_transition_to_invalid() {
        // Order::transition_to() が無効な遷移で OrderError を返すことを確認する
        let order = Order {
            id: uuid::Uuid::new_v4(),
            customer_id: "CUST-001".to_string(),
            status: OrderStatus::Delivered,
            total_amount: 1000,
            currency: "JPY".to_string(),
            notes: None,
            created_by: "admin".to_string(),
            updated_by: None,
            version: 1,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let result = order.transition_to(OrderStatus::Pending);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("invalid status transition"));
    }

    #[test]
    fn test_create_order_validate_success() {
        // CreateOrder::validate() が有効な入力で Ok を返すことを確認する
        let order = CreateOrder {
            customer_id: "CUST-001".to_string(),
            currency: "JPY".to_string(),
            notes: None,
            items: vec![CreateOrderItem {
                product_id: "PROD-001".to_string(),
                product_name: "Widget".to_string(),
                quantity: 2,
                unit_price: 500,
            }],
        };
        assert!(order.validate().is_ok());
    }

    #[test]
    fn test_create_order_validate_empty_items() {
        // CreateOrder::validate() が空の items で ValidationFailed を返すことを確認する
        let order = CreateOrder {
            customer_id: "CUST-001".to_string(),
            currency: "JPY".to_string(),
            notes: None,
            items: vec![],
        };
        let result = order.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least one item"));
    }

    #[test]
    fn test_create_order_calculate_total() {
        // CreateOrder::calculate_total() が正しい合計を返すことを確認する
        let order = CreateOrder {
            customer_id: "CUST-001".to_string(),
            currency: "JPY".to_string(),
            notes: None,
            items: vec![
                CreateOrderItem {
                    product_id: "A".to_string(),
                    product_name: "A".to_string(),
                    quantity: 2,
                    unit_price: 1000,
                },
                CreateOrderItem {
                    product_id: "B".to_string(),
                    product_name: "B".to_string(),
                    quantity: 3,
                    unit_price: 500,
                },
            ],
        };
        assert_eq!(order.calculate_total(), 3500);
    }
}
