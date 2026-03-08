use crate::domain::entity::order::{Order, OrderItem};
use crate::domain::repository::order_repository::OrderRepository;
use std::sync::Arc;
use uuid::Uuid;

pub struct GetOrderUseCase {
    order_repo: Arc<dyn OrderRepository>,
}

impl GetOrderUseCase {
    pub fn new(order_repo: Arc<dyn OrderRepository>) -> Self {
        Self { order_repo }
    }

    pub async fn execute(&self, order_id: Uuid) -> anyhow::Result<(Order, Vec<OrderItem>)> {
        let order = self
            .order_repo
            .find_by_id(order_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Order '{}' not found", order_id))?;

        let items = self.order_repo.find_items_by_order_id(order_id).await?;

        Ok((order, items))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::order::{OrderStatus};
    use crate::domain::repository::order_repository::MockOrderRepository;
    use chrono::Utc;

    #[tokio::test]
    async fn test_get_order_success() {
        let order_id = Uuid::new_v4();
        let order = Order {
            id: order_id,
            customer_id: "CUST-001".to_string(),
            status: OrderStatus::Pending,
            total_amount: 2000,
            currency: "JPY".to_string(),
            notes: None,
            created_by: "admin".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let items = vec![OrderItem {
            id: Uuid::new_v4(),
            order_id,
            product_id: "PROD-001".to_string(),
            product_name: "Widget".to_string(),
            quantity: 2,
            unit_price: 1000,
            subtotal: 2000,
            created_at: Utc::now(),
        }];
        let order_clone = order.clone();
        let items_clone = items.clone();

        let mut mock_repo = MockOrderRepository::new();
        mock_repo
            .expect_find_by_id()
            .withf(move |id| *id == order_id)
            .times(1)
            .returning(move |_| Ok(Some(order_clone.clone())));
        mock_repo
            .expect_find_items_by_order_id()
            .withf(move |id| *id == order_id)
            .times(1)
            .returning(move |_| Ok(items_clone.clone()));

        let uc = GetOrderUseCase::new(Arc::new(mock_repo));
        let result = uc.execute(order_id).await;
        assert!(result.is_ok());
        let (found_order, found_items) = result.unwrap();
        assert_eq!(found_order.id, order_id);
        assert_eq!(found_items.len(), 1);
    }

    #[tokio::test]
    async fn test_get_order_not_found() {
        let order_id = Uuid::new_v4();
        let mut mock_repo = MockOrderRepository::new();
        mock_repo
            .expect_find_by_id()
            .times(1)
            .returning(|_| Ok(None));

        let uc = GetOrderUseCase::new(Arc::new(mock_repo));
        let result = uc.execute(order_id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}
