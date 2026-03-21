use crate::domain::entity::order::{CreateOrder, Order, OrderStatus};
use crate::domain::error::OrderError;

/// 注文ドメインサービス — 複数エンティティにまたがるドメインルールを担当。
/// 単一エンティティ内で完結するバリデーション・計算は Order / CreateOrder メソッドに委譲する（H-005 / M-001）。
pub struct OrderDomainService;

impl OrderDomainService {
    /// 注文作成入力を検証する。
    /// 実装は CreateOrder::validate() に委譲する（ドメインルールのエンティティへの集約）。
    pub fn validate_create_order(input: &CreateOrder) -> Result<(), OrderError> {
        input.validate()
    }

    /// ステータス遷移を検証する。
    /// 実装は Order::transition_to() に委譲する（ドメインルールのエンティティへの集約）。
    pub fn validate_status_transition(
        order: &Order,
        next: &OrderStatus,
    ) -> Result<(), OrderError> {
        order.transition_to(next.clone()).map(|_| ())
    }

    /// 注文明細から合計金額を計算する。
    /// 実装は CreateOrder::calculate_total() に委譲する（ドメインルールのエンティティへの集約）。
    pub fn calculate_total(input: &CreateOrder) -> i64 {
        input.calculate_total()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::order::{CreateOrder, CreateOrderItem};
    use chrono::Utc;
    use uuid::Uuid;

    fn valid_create_order() -> CreateOrder {
        CreateOrder {
            customer_id: "CUST-001".to_string(),
            currency: "JPY".to_string(),
            notes: None,
            items: vec![CreateOrderItem {
                product_id: "PROD-001".to_string(),
                product_name: "Widget".to_string(),
                quantity: 2,
                unit_price: 500,
            }],
        }
    }

    fn sample_order(status: OrderStatus) -> Order {
        Order {
            id: Uuid::new_v4(),
            customer_id: "CUST-001".to_string(),
            status,
            total_amount: 1000,
            currency: "JPY".to_string(),
            notes: None,
            created_by: "admin".to_string(),
            updated_by: None,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_validate_create_order_success() {
        let input = valid_create_order();
        assert!(OrderDomainService::validate_create_order(&input).is_ok());
    }

    #[test]
    fn test_validate_create_order_empty_customer() {
        let mut input = valid_create_order();
        input.customer_id = "".to_string();
        let result = OrderDomainService::validate_create_order(&input);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("customer_id"));
    }

    #[test]
    fn test_validate_create_order_no_items() {
        let mut input = valid_create_order();
        input.items = vec![];
        let result = OrderDomainService::validate_create_order(&input);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("at least one item"));
    }

    #[test]
    fn test_validate_create_order_zero_quantity() {
        let mut input = valid_create_order();
        input.items[0].quantity = 0;
        let result = OrderDomainService::validate_create_order(&input);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("quantity"));
    }

    #[test]
    fn test_calculate_total() {
        // CreateOrder::calculate_total() への委譲を確認する
        let input = CreateOrder {
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
        assert_eq!(OrderDomainService::calculate_total(&input), 3500);
    }

    #[test]
    fn test_valid_status_transition() {
        let order = sample_order(OrderStatus::Pending);
        assert!(
            OrderDomainService::validate_status_transition(&order, &OrderStatus::Confirmed)
                .is_ok()
        );
    }

    #[test]
    fn test_invalid_status_transition() {
        let order = sample_order(OrderStatus::Delivered);
        let result =
            OrderDomainService::validate_status_transition(&order, &OrderStatus::Pending);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("invalid status transition"));
    }
}
