use async_trait::async_trait;

use crate::envelope::EventEnvelope;
use crate::error::PublishError;

/// イベントを発行する trait。
#[async_trait]
pub trait EventPublisher: Send + Sync {
    /// エンベロープをイベントバスに発行する。
    async fn publish(&self, envelope: EventEnvelope) -> Result<(), PublishError>;

    /// 複数のエンベロープをバッチ発行する。
    ///
    /// デフォルト実装は逐次発行する。
    async fn publish_batch(&self, envelopes: Vec<EventEnvelope>) -> Result<(), PublishError> {
        for envelope in envelopes {
            self.publish(envelope).await?;
        }
        Ok(())
    }
}
