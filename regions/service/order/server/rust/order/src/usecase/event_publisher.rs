// 注文イベントパブリッシャートレイト。
// usecase層はドメインイベント型のみに依存し、Proto型には依存しない。
// Proto型への変換はインフラ層（Kafka producer）で行う。

use crate::domain::entity::event::{
    OrderCancelledDomainEvent, OrderCreatedDomainEvent, OrderUpdatedDomainEvent,
};
use async_trait::async_trait;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait OrderEventPublisher: Send + Sync {
    // 注文作成イベントをドメインイベント型で publish する
    async fn publish_order_created(&self, event: &OrderCreatedDomainEvent) -> anyhow::Result<()>;
    // 注文更新イベントをドメインイベント型で publish する
    async fn publish_order_updated(&self, event: &OrderUpdatedDomainEvent) -> anyhow::Result<()>;
    // 注文キャンセルイベントをドメインイベント型で publish する
    async fn publish_order_cancelled(
        &self,
        event: &OrderCancelledDomainEvent,
    ) -> anyhow::Result<()>;
}

// イベント publish を行わない Noop 実装（テスト・ローカル開発用）
pub struct NoopOrderEventPublisher;

#[async_trait]
impl OrderEventPublisher for NoopOrderEventPublisher {
    async fn publish_order_created(&self, _event: &OrderCreatedDomainEvent) -> anyhow::Result<()> {
        Ok(())
    }

    async fn publish_order_updated(&self, _event: &OrderUpdatedDomainEvent) -> anyhow::Result<()> {
        Ok(())
    }

    async fn publish_order_cancelled(
        &self,
        _event: &OrderCancelledDomainEvent,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_noop_publisher_order_created() {
        let publisher = NoopOrderEventPublisher;
        // デフォルト値のドメインイベントで publish が成功することを検証
        let event = OrderCreatedDomainEvent::default();
        assert!(publisher.publish_order_created(&event).await.is_ok());
    }

    #[tokio::test]
    async fn test_noop_publisher_order_updated() {
        let publisher = NoopOrderEventPublisher;
        let event = OrderUpdatedDomainEvent::default();
        assert!(publisher.publish_order_updated(&event).await.is_ok());
    }

    #[tokio::test]
    async fn test_noop_publisher_order_cancelled() {
        let publisher = NoopOrderEventPublisher;
        let event = OrderCancelledDomainEvent::default();
        assert!(publisher.publish_order_cancelled(&event).await.is_ok());
    }
}
