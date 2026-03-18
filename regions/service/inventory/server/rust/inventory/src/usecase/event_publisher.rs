// 在庫イベントパブリッシャートレイト。
// usecase層はドメインイベント型のみに依存し、Proto型には依存しない。
// Proto型への変換はインフラ層（Kafka producer）で行う。

use crate::domain::entity::event::{InventoryReleasedDomainEvent, InventoryReservedDomainEvent};
use async_trait::async_trait;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait InventoryEventPublisher: Send + Sync {
    // 在庫予約イベントをドメインイベント型で publish する
    async fn publish_inventory_reserved(
        &self,
        event: &InventoryReservedDomainEvent,
    ) -> anyhow::Result<()>;
    // 在庫解放イベントをドメインイベント型で publish する
    async fn publish_inventory_released(
        &self,
        event: &InventoryReleasedDomainEvent,
    ) -> anyhow::Result<()>;
}

// イベント未送信時のスタブ実装
pub struct NoopInventoryEventPublisher;

#[async_trait]
impl InventoryEventPublisher for NoopInventoryEventPublisher {
    async fn publish_inventory_reserved(
        &self,
        _event: &InventoryReservedDomainEvent,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn publish_inventory_released(
        &self,
        _event: &InventoryReleasedDomainEvent,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_noop_publisher_inventory_reserved() {
        let publisher = NoopInventoryEventPublisher;
        let event = InventoryReservedDomainEvent::default();
        assert!(publisher.publish_inventory_reserved(&event).await.is_ok());
    }

    #[tokio::test]
    async fn test_noop_publisher_inventory_released() {
        let publisher = NoopInventoryEventPublisher;
        let event = InventoryReleasedDomainEvent::default();
        assert!(publisher.publish_inventory_released(&event).await.is_ok());
    }
}
