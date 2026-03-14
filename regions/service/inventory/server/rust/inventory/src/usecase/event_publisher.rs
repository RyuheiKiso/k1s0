use async_trait::async_trait;
use serde_json::Value;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait InventoryEventPublisher: Send + Sync {
    async fn publish_inventory_reserved(&self, event: &Value) -> anyhow::Result<()>;
    async fn publish_inventory_released(&self, event: &Value) -> anyhow::Result<()>;
}

pub struct NoopInventoryEventPublisher;

#[async_trait]
impl InventoryEventPublisher for NoopInventoryEventPublisher {
    async fn publish_inventory_reserved(&self, _event: &Value) -> anyhow::Result<()> {
        Ok(())
    }

    async fn publish_inventory_released(&self, _event: &Value) -> anyhow::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_noop_publisher_inventory_reserved() {
        let publisher = NoopInventoryEventPublisher;
        let event = serde_json::json!({"event_type": "inventory.reserved"});
        assert!(publisher.publish_inventory_reserved(&event).await.is_ok());
    }

    #[tokio::test]
    async fn test_noop_publisher_inventory_released() {
        let publisher = NoopInventoryEventPublisher;
        let event = serde_json::json!({"event_type": "inventory.released"});
        assert!(publisher.publish_inventory_released(&event).await.is_ok());
    }
}
