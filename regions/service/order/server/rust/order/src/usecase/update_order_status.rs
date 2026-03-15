use crate::domain::entity::order::{Order, OrderStatus};
use crate::domain::error::OrderError;
use crate::domain::repository::order_repository::OrderRepository;
use crate::domain::service::order_service::OrderDomainService;
use std::sync::Arc;
use uuid::Uuid;

pub struct UpdateOrderStatusUseCase {
    order_repo: Arc<dyn OrderRepository>,
}

impl UpdateOrderStatusUseCase {
    pub fn new(order_repo: Arc<dyn OrderRepository>) -> Self {
        Self { order_repo }
    }

    pub async fn execute(
        &self,
        order_id: Uuid,
        new_status: &OrderStatus,
        actor: &str,
    ) -> anyhow::Result<Order> {
        let existing = self
            .order_repo
            .find_by_id(order_id)
            .await?
            .ok_or_else(|| OrderError::NotFound(order_id.to_string()))?;

        // ステータス遷移バリデーション
        OrderDomainService::validate_status_transition(&existing.status, new_status)?;

        // ステータス更新（Outbox イベントも同一トランザクション内で挿入される）
        let updated = self
            .order_repo
            .update_status(order_id, new_status, actor, existing.version)
            .await?;

        Ok(updated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::order_repository::MockOrderRepository;
    use chrono::Utc;

    fn sample_order(status: OrderStatus) -> Order {
        Order {
            id: Uuid::new_v4(),
            customer_id: "CUST-001".to_string(),
            status,
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

    #[tokio::test]
    async fn test_update_status_success() {
        let order = sample_order(OrderStatus::Pending);
        let order_id = order.id;
        let mut confirmed_order = order.clone();
        confirmed_order.status = OrderStatus::Confirmed;
        let order_clone = order.clone();
        let confirmed_clone = confirmed_order.clone();

        let mut mock_repo = MockOrderRepository::new();

        mock_repo
            .expect_find_by_id()
            .withf(move |id| *id == order_id)
            .times(1)
            .returning(move |_| Ok(Some(order_clone.clone())));

        mock_repo
            .expect_update_status()
            .times(1)
            .returning(move |_, _, _, _| Ok(confirmed_clone.clone()));

        let uc = UpdateOrderStatusUseCase::new(Arc::new(mock_repo));
        let result = uc.execute(order_id, &OrderStatus::Confirmed, "admin").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, OrderStatus::Confirmed);
    }

    #[tokio::test]
    async fn test_update_status_invalid_transition() {
        let order = sample_order(OrderStatus::Delivered);
        let order_id = order.id;
        let order_clone = order.clone();

        let mut mock_repo = MockOrderRepository::new();

        mock_repo
            .expect_find_by_id()
            .times(1)
            .returning(move |_| Ok(Some(order_clone.clone())));

        let uc = UpdateOrderStatusUseCase::new(Arc::new(mock_repo));
        let result = uc.execute(order_id, &OrderStatus::Pending, "admin").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("invalid status transition"));
    }

    #[tokio::test]
    async fn test_update_status_cancel() {
        let order = sample_order(OrderStatus::Pending);
        let order_id = order.id;
        let mut cancelled_order = order.clone();
        cancelled_order.status = OrderStatus::Cancelled;
        let order_clone = order.clone();
        let cancelled_clone = cancelled_order.clone();

        let mut mock_repo = MockOrderRepository::new();

        mock_repo
            .expect_find_by_id()
            .times(1)
            .returning(move |_| Ok(Some(order_clone.clone())));
        mock_repo
            .expect_update_status()
            .times(1)
            .returning(move |_, _, _, _| Ok(cancelled_clone.clone()));

        let uc = UpdateOrderStatusUseCase::new(Arc::new(mock_repo));
        let result = uc.execute(order_id, &OrderStatus::Cancelled, "admin").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, OrderStatus::Cancelled);
    }
}
