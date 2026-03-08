use crate::domain::entity::order::{Order, OrderFilter};
use crate::domain::repository::order_repository::OrderRepository;
use std::sync::Arc;

pub struct ListOrdersUseCase {
    order_repo: Arc<dyn OrderRepository>,
}

impl ListOrdersUseCase {
    pub fn new(order_repo: Arc<dyn OrderRepository>) -> Self {
        Self { order_repo }
    }

    pub async fn execute(&self, filter: &OrderFilter) -> anyhow::Result<(Vec<Order>, i64)> {
        let orders = self.order_repo.find_all(filter).await?;
        let total = self.order_repo.count(filter).await?;
        Ok((orders, total))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::order::OrderStatus;
    use crate::domain::repository::order_repository::MockOrderRepository;
    use chrono::Utc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_list_orders_success() {
        let orders = vec![Order {
            id: Uuid::new_v4(),
            customer_id: "CUST-001".to_string(),
            status: OrderStatus::Pending,
            total_amount: 2000,
            currency: "JPY".to_string(),
            notes: None,
            created_by: "admin".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }];
        let orders_clone = orders.clone();

        let mut mock_repo = MockOrderRepository::new();
        mock_repo
            .expect_find_all()
            .times(1)
            .returning(move |_| Ok(orders_clone.clone()));
        mock_repo
            .expect_count()
            .times(1)
            .returning(|_| Ok(1));

        let uc = ListOrdersUseCase::new(Arc::new(mock_repo));
        let filter = OrderFilter::default();
        let result = uc.execute(&filter).await;
        assert!(result.is_ok());
        let (found_orders, total) = result.unwrap();
        assert_eq!(found_orders.len(), 1);
        assert_eq!(total, 1);
    }

    #[tokio::test]
    async fn test_list_orders_empty() {
        let mut mock_repo = MockOrderRepository::new();
        mock_repo
            .expect_find_all()
            .times(1)
            .returning(|_| Ok(vec![]));
        mock_repo
            .expect_count()
            .times(1)
            .returning(|_| Ok(0));

        let uc = ListOrdersUseCase::new(Arc::new(mock_repo));
        let filter = OrderFilter::default();
        let result = uc.execute(&filter).await;
        assert!(result.is_ok());
        let (found_orders, total) = result.unwrap();
        assert_eq!(found_orders.len(), 0);
        assert_eq!(total, 0);
    }
}
