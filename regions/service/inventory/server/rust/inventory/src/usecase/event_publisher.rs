use async_trait::async_trait;
use crate::proto::k1s0::event::service::inventory::v1::{
    InventoryReservedEvent, InventoryReleasedEvent,
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait InventoryEventPublisher: Send + Sync {
    // 在庫予約イベントを Protobuf 形式で Kafka に publish する
    async fn publish_inventory_reserved(&self, event: &InventoryReservedEvent) -> anyhow::Result<()>;
    // 在庫解放イベントを Protobuf 形式で Kafka に publish する
    async fn publish_inventory_released(&self, event: &InventoryReleasedEvent) -> anyhow::Result<()>;
}

// イベント未送信時のスタブ実装
pub struct NoopInventoryEventPublisher;

#[async_trait]
impl InventoryEventPublisher for NoopInventoryEventPublisher {
    async fn publish_inventory_reserved(&self, _event: &InventoryReservedEvent) -> anyhow::Result<()> {
        Ok(())
    }

    async fn publish_inventory_released(&self, _event: &InventoryReleasedEvent) -> anyhow::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_noop_publisher_inventory_reserved() {
        let publisher = NoopInventoryEventPublisher;
        let event = InventoryReservedEvent::default();
        assert!(publisher.publish_inventory_reserved(&event).await.is_ok());
    }

    #[tokio::test]
    async fn test_noop_publisher_inventory_released() {
        let publisher = NoopInventoryEventPublisher;
        let event = InventoryReleasedEvent::default();
        assert!(publisher.publish_inventory_released(&event).await.is_ok());
    }
}
