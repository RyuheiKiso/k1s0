use async_trait::async_trait;
use serde_json::Value;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait OrderEventPublisher: Send + Sync {
    async fn publish_order_created(&self, event: &Value) -> anyhow::Result<()>;
    async fn publish_order_updated(&self, event: &Value) -> anyhow::Result<()>;
    async fn publish_order_cancelled(&self, event: &Value) -> anyhow::Result<()>;
}

pub struct NoopOrderEventPublisher;

#[async_trait]
impl OrderEventPublisher for NoopOrderEventPublisher {
    async fn publish_order_created(&self, _event: &Value) -> anyhow::Result<()> {
        Ok(())
    }

    async fn publish_order_updated(&self, _event: &Value) -> anyhow::Result<()> {
        Ok(())
    }

    async fn publish_order_cancelled(&self, _event: &Value) -> anyhow::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_noop_publisher_order_created() {
        let publisher = NoopOrderEventPublisher;
        let event = serde_json::json!({"event_type": "order.created"});
        assert!(publisher.publish_order_created(&event).await.is_ok());
    }

    #[tokio::test]
    async fn test_noop_publisher_order_updated() {
        let publisher = NoopOrderEventPublisher;
        let event = serde_json::json!({"event_type": "order.updated"});
        assert!(publisher.publish_order_updated(&event).await.is_ok());
    }

    #[tokio::test]
    async fn test_noop_publisher_order_cancelled() {
        let publisher = NoopOrderEventPublisher;
        let event = serde_json::json!({"event_type": "order.cancelled"});
        assert!(publisher.publish_order_cancelled(&event).await.is_ok());
    }
}
