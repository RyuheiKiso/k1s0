use async_trait::async_trait;

use crate::error::MessagingError;
use crate::event::EventEnvelope;

/// EventProducer は Kafka イベントの発行インターフェース。
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait EventProducer: Send + Sync {
    /// 単一のイベントを発行する。
    async fn publish(&self, envelope: EventEnvelope) -> Result<(), MessagingError>;

    /// 複数のイベントをバッチで発行する。
    async fn publish_batch(&self, envelopes: Vec<EventEnvelope>) -> Result<(), MessagingError>;
}

/// NoOpEventProducer はテスト・スタブ用の何もしないプロデューサー実装。
pub struct NoOpEventProducer;

#[async_trait]
impl EventProducer for NoOpEventProducer {
    async fn publish(&self, _envelope: EventEnvelope) -> Result<(), MessagingError> {
        Ok(())
    }

    async fn publish_batch(&self, _envelopes: Vec<EventEnvelope>) -> Result<(), MessagingError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::EventEnvelope;

    #[tokio::test]
    async fn test_noop_producer_publish() {
        let producer = NoOpEventProducer;
        let envelope = EventEnvelope {
            topic: "test.topic".to_string(),
            key: "key-1".to_string(),
            payload: b"test payload".to_vec(),
            headers: vec![],
        };
        let result = producer.publish(envelope).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_noop_producer_publish_batch() {
        let producer = NoOpEventProducer;
        let envelopes = vec![
            EventEnvelope {
                topic: "test.topic".to_string(),
                key: "key-1".to_string(),
                payload: b"payload 1".to_vec(),
                headers: vec![],
            },
            EventEnvelope {
                topic: "test.topic".to_string(),
                key: "key-2".to_string(),
                payload: b"payload 2".to_vec(),
                headers: vec![],
            },
        ];
        let result = producer.publish_batch(envelopes).await;
        assert!(result.is_ok());
    }
}
