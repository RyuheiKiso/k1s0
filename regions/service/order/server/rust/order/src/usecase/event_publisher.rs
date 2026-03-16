use async_trait::async_trait;
use crate::proto::k1s0::event::service::order::v1::{
    OrderCreatedEvent, OrderUpdatedEvent, OrderCancelledEvent,
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait OrderEventPublisher: Send + Sync {
    // 注文作成イベントを Protobuf 形式で Kafka に publish する
    async fn publish_order_created(&self, event: &OrderCreatedEvent) -> anyhow::Result<()>;
    // 注文更新イベントを Protobuf 形式で Kafka に publish する
    async fn publish_order_updated(&self, event: &OrderUpdatedEvent) -> anyhow::Result<()>;
    // 注文キャンセルイベントを Protobuf 形式で Kafka に publish する
    async fn publish_order_cancelled(&self, event: &OrderCancelledEvent) -> anyhow::Result<()>;
}

// イベント publish を行わない Noop 実装（テスト・ローカル開発用）
pub struct NoopOrderEventPublisher;

#[async_trait]
impl OrderEventPublisher for NoopOrderEventPublisher {
    async fn publish_order_created(&self, _event: &OrderCreatedEvent) -> anyhow::Result<()> {
        Ok(())
    }

    async fn publish_order_updated(&self, _event: &OrderUpdatedEvent) -> anyhow::Result<()> {
        Ok(())
    }

    async fn publish_order_cancelled(&self, _event: &OrderCancelledEvent) -> anyhow::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_noop_publisher_order_created() {
        let publisher = NoopOrderEventPublisher;
        // デフォルト値の Protobuf メッセージで publish が成功することを検証
        let event = OrderCreatedEvent::default();
        assert!(publisher.publish_order_created(&event).await.is_ok());
    }

    #[tokio::test]
    async fn test_noop_publisher_order_updated() {
        let publisher = NoopOrderEventPublisher;
        let event = OrderUpdatedEvent::default();
        assert!(publisher.publish_order_updated(&event).await.is_ok());
    }

    #[tokio::test]
    async fn test_noop_publisher_order_cancelled() {
        let publisher = NoopOrderEventPublisher;
        let event = OrderCancelledEvent::default();
        assert!(publisher.publish_order_cancelled(&event).await.is_ok());
    }
}
