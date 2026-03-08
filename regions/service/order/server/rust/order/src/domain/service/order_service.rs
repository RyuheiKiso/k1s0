use crate::domain::entity::order::{CreateOrder, OrderStatus};

/// 注文ドメインサービス — ドメインルールのバリデーションを担当。
pub struct OrderDomainService;

impl OrderDomainService {
    /// 注文作成入力を検証する。
    pub fn validate_create_order(input: &CreateOrder) -> anyhow::Result<()> {
        if input.customer_id.trim().is_empty() {
            anyhow::bail!("customer_id must not be empty");
        }
        if input.currency.trim().is_empty() {
            anyhow::bail!("currency must not be empty");
        }
        if input.items.is_empty() {
            anyhow::bail!("order must contain at least one item");
        }
        for (i, item) in input.items.iter().enumerate() {
            if item.product_id.trim().is_empty() {
                anyhow::bail!("items[{}].product_id must not be empty", i);
            }
            if item.product_name.trim().is_empty() {
                anyhow::bail!("items[{}].product_name must not be empty", i);
            }
            if item.quantity <= 0 {
                anyhow::bail!("items[{}].quantity must be greater than zero", i);
            }
            if item.unit_price < 0 {
                anyhow::bail!("items[{}].unit_price must not be negative", i);
            }
        }
        Ok(())
    }

    /// ステータス遷移を検証する。
    pub fn validate_status_transition(
        current: &OrderStatus,
        next: &OrderStatus,
    ) -> anyhow::Result<()> {
        if !current.can_transition_to(next) {
            anyhow::bail!(
                "invalid status transition: '{}' -> '{}'",
                current,
                next
            );
        }
        Ok(())
    }

    /// 注文明細から合計金額を計算する。
    pub fn calculate_total(items: &[crate::domain::entity::order::CreateOrderItem]) -> i64 {
        items
            .iter()
            .map(|item| item.quantity as i64 * item.unit_price)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::order::{CreateOrder, CreateOrderItem};

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
        assert!(result.unwrap_err().to_string().contains("at least one item"));
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
        let items = vec![
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
        ];
        assert_eq!(OrderDomainService::calculate_total(&items), 3500);
    }

    #[test]
    fn test_valid_status_transition() {
        assert!(OrderDomainService::validate_status_transition(
            &OrderStatus::Pending,
            &OrderStatus::Confirmed,
        )
        .is_ok());
    }

    #[test]
    fn test_invalid_status_transition() {
        let result = OrderDomainService::validate_status_transition(
            &OrderStatus::Delivered,
            &OrderStatus::Pending,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid status transition"));
    }
}
