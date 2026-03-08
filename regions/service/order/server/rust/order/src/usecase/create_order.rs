use crate::domain::entity::order::{CreateOrder, Order, OrderItem};
use crate::domain::repository::order_repository::OrderRepository;
use crate::domain::service::order_service::OrderDomainService;
use std::sync::Arc;

pub struct CreateOrderUseCase {
    order_repo: Arc<dyn OrderRepository>,
}

impl CreateOrderUseCase {
    pub fn new(order_repo: Arc<dyn OrderRepository>) -> Self {
        Self { order_repo }
    }

    pub async fn execute(
        &self,
        input: &CreateOrder,
        created_by: &str,
    ) -> anyhow::Result<(Order, Vec<OrderItem>)> {
        // ドメインバリデーション（OrderError を返す）
        OrderDomainService::validate_create_order(input)?;

        // 永続化（Outbox イベントも同一トランザクション内で挿入される）
        let (order, items) = self.order_repo.create(input, created_by).await?;

        Ok((order, items))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::order::{CreateOrderItem, OrderStatus};
    use crate::domain::repository::order_repository::MockOrderRepository;
    use chrono::Utc;
    use uuid::Uuid;

    fn sample_create_order() -> CreateOrder {
        CreateOrder {
            customer_id: "CUST-001".to_string(),
            currency: "JPY".to_string(),
            notes: None,
            items: vec![CreateOrderItem {
                product_id: "PROD-001".to_string(),
                product_name: "Widget".to_string(),
                quantity: 2,
                unit_price: 1000,
            }],
        }
    }

    fn sample_order() -> Order {
        Order {
            id: Uuid::new_v4(),
            customer_id: "CUST-001".to_string(),
            status: OrderStatus::Pending,
            total_amount: 2000,
            currency: "JPY".to_string(),
            notes: None,
            created_by: "admin".to_string(),
            updated_by: None,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn sample_order_items(order_id: Uuid) -> Vec<OrderItem> {
        vec![OrderItem {
            id: Uuid::new_v4(),
            order_id,
            product_id: "PROD-001".to_string(),
            product_name: "Widget".to_string(),
            quantity: 2,
            unit_price: 1000,
            subtotal: 2000,
            created_at: Utc::now(),
        }]
    }

    #[tokio::test]
    async fn test_create_order_success() {
        let order = sample_order();
        let order_id = order.id;
        let items = sample_order_items(order_id);
        let order_clone = order.clone();
        let items_clone = items.clone();

        let mut mock_repo = MockOrderRepository::new();

        mock_repo
            .expect_create()
            .times(1)
            .returning(move |_, _| Ok((order_clone.clone(), items_clone.clone())));

        let uc = CreateOrderUseCase::new(Arc::new(mock_repo));
        let input = sample_create_order();
        let result = uc.execute(&input, "admin").await;
        assert!(result.is_ok());
        let (created_order, created_items) = result.unwrap();
        assert_eq!(created_order.customer_id, "CUST-001");
        assert_eq!(created_items.len(), 1);
    }

    #[tokio::test]
    async fn test_create_order_validation_failure() {
        let mock_repo = MockOrderRepository::new();

        let uc = CreateOrderUseCase::new(Arc::new(mock_repo));
        let input = CreateOrder {
            customer_id: "".to_string(),
            currency: "JPY".to_string(),
            notes: None,
            items: vec![],
        };
        let result = uc.execute(&input, "admin").await;
        assert!(result.is_err());
    }
}
