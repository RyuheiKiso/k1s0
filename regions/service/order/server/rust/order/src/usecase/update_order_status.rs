use crate::domain::entity::order::{Order, OrderStatus};
use crate::domain::repository::order_repository::OrderRepository;
use crate::domain::service::order_service::OrderDomainService;
use crate::usecase::event_publisher::OrderEventPublisher;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub struct UpdateOrderStatusUseCase {
    order_repo: Arc<dyn OrderRepository>,
    event_publisher: Arc<dyn OrderEventPublisher>,
}

impl UpdateOrderStatusUseCase {
    pub fn new(
        order_repo: Arc<dyn OrderRepository>,
        event_publisher: Arc<dyn OrderEventPublisher>,
    ) -> Self {
        Self {
            order_repo,
            event_publisher,
        }
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
            .ok_or_else(|| anyhow::anyhow!("Order '{}' not found", order_id))?;

        // ステータス遷移バリデーション
        OrderDomainService::validate_status_transition(&existing.status, new_status)?;

        let updated = self.order_repo.update_status(order_id, new_status).await?;

        // イベント発行
        if *new_status == OrderStatus::Cancelled {
            self.publish_cancelled_event(&updated, actor).await;
        } else {
            self.publish_updated_event(&updated, actor).await;
        }

        Ok(updated)
    }

    async fn publish_updated_event(&self, order: &Order, actor: &str) {
        let event = serde_json::json!({
            "event_id": Uuid::new_v4().to_string(),
            "event_type": "order.updated",
            "order_id": order.id.to_string(),
            "user_id": actor,
            "status": order.status.as_str(),
            "total_amount": order.total_amount,
            "timestamp": Utc::now().to_rfc3339(),
        });

        if let Err(err) = self.event_publisher.publish_order_updated(&event).await {
            tracing::warn!(
                error = %err,
                order_id = %order.id,
                "failed to publish order updated event"
            );
        }
    }

    async fn publish_cancelled_event(&self, order: &Order, actor: &str) {
        let event = serde_json::json!({
            "event_id": Uuid::new_v4().to_string(),
            "event_type": "order.cancelled",
            "order_id": order.id.to_string(),
            "user_id": actor,
            "reason": "status changed to cancelled",
            "timestamp": Utc::now().to_rfc3339(),
        });

        if let Err(err) = self.event_publisher.publish_order_cancelled(&event).await {
            tracing::warn!(
                error = %err,
                order_id = %order.id,
                "failed to publish order cancelled event"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::order_repository::MockOrderRepository;
    use crate::usecase::event_publisher::MockOrderEventPublisher;
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
        let mut mock_publisher = MockOrderEventPublisher::new();

        mock_repo
            .expect_find_by_id()
            .withf(move |id| *id == order_id)
            .times(1)
            .returning(move |_| Ok(Some(order_clone.clone())));

        mock_repo
            .expect_update_status()
            .times(1)
            .returning(move |_, _| Ok(confirmed_clone.clone()));

        mock_publisher
            .expect_publish_order_updated()
            .times(1)
            .returning(|_| Ok(()));

        let uc = UpdateOrderStatusUseCase::new(Arc::new(mock_repo), Arc::new(mock_publisher));
        let result = uc
            .execute(order_id, &OrderStatus::Confirmed, "admin")
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, OrderStatus::Confirmed);
    }

    #[tokio::test]
    async fn test_update_status_invalid_transition() {
        let order = sample_order(OrderStatus::Delivered);
        let order_id = order.id;
        let order_clone = order.clone();

        let mut mock_repo = MockOrderRepository::new();
        let mock_publisher = MockOrderEventPublisher::new();

        mock_repo
            .expect_find_by_id()
            .times(1)
            .returning(move |_| Ok(Some(order_clone.clone())));

        let uc = UpdateOrderStatusUseCase::new(Arc::new(mock_repo), Arc::new(mock_publisher));
        let result = uc
            .execute(order_id, &OrderStatus::Pending, "admin")
            .await;
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
        let mut mock_publisher = MockOrderEventPublisher::new();

        mock_repo
            .expect_find_by_id()
            .times(1)
            .returning(move |_| Ok(Some(order_clone.clone())));
        mock_repo
            .expect_update_status()
            .times(1)
            .returning(move |_, _| Ok(cancelled_clone.clone()));
        mock_publisher
            .expect_publish_order_cancelled()
            .times(1)
            .returning(|_| Ok(()));

        let uc = UpdateOrderStatusUseCase::new(Arc::new(mock_repo), Arc::new(mock_publisher));
        let result = uc
            .execute(order_id, &OrderStatus::Cancelled, "admin")
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, OrderStatus::Cancelled);
    }
}
