use crate::domain::entity::order::{CreateOrder, Order, OrderItem};
use crate::domain::repository::order_repository::OrderRepository;
use crate::domain::service::order_service::OrderDomainService;
use crate::usecase::event_publisher::OrderEventPublisher;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub struct CreateOrderUseCase {
    order_repo: Arc<dyn OrderRepository>,
    event_publisher: Arc<dyn OrderEventPublisher>,
}

impl CreateOrderUseCase {
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
        input: &CreateOrder,
        created_by: &str,
    ) -> anyhow::Result<(Order, Vec<OrderItem>)> {
        // ドメインバリデーション
        OrderDomainService::validate_create_order(input)?;

        // 永続化
        let (order, items) = self.order_repo.create(input, created_by).await?;

        // イベント発行
        self.publish_created_event(&order, &items).await;

        Ok((order, items))
    }

    async fn publish_created_event(&self, order: &Order, items: &[OrderItem]) {
        let items_json: Vec<serde_json::Value> = items
            .iter()
            .map(|item| {
                serde_json::json!({
                    "product_id": item.product_id,
                    "quantity": item.quantity,
                    "unit_price": item.unit_price,
                })
            })
            .collect();

        let event = serde_json::json!({
            "event_id": Uuid::new_v4().to_string(),
            "event_type": "order.created",
            "order_id": order.id.to_string(),
            "customer_id": order.customer_id,
            "items": items_json,
            "total_amount": order.total_amount,
            "currency": order.currency,
            "timestamp": Utc::now().to_rfc3339(),
        });

        if let Err(err) = self.event_publisher.publish_order_created(&event).await {
            tracing::warn!(
                error = %err,
                order_id = %order.id,
                "failed to publish order created event"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::order::{CreateOrderItem, OrderStatus};
    use crate::domain::repository::order_repository::MockOrderRepository;
    use crate::usecase::event_publisher::MockOrderEventPublisher;
    use chrono::Utc;

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
        let mut mock_publisher = MockOrderEventPublisher::new();

        mock_repo
            .expect_create()
            .times(1)
            .returning(move |_, _| Ok((order_clone.clone(), items_clone.clone())));

        mock_publisher
            .expect_publish_order_created()
            .times(1)
            .returning(|_| Ok(()));

        let uc = CreateOrderUseCase::new(Arc::new(mock_repo), Arc::new(mock_publisher));
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
        let mock_publisher = MockOrderEventPublisher::new();

        let uc = CreateOrderUseCase::new(Arc::new(mock_repo), Arc::new(mock_publisher));
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
