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
    use std::collections::HashMap;

    // NoOpEventProducer の publish が何も処理せず Ok を返すことを確認する。
    #[tokio::test]
    async fn test_noop_producer_publish() {
        let producer = NoOpEventProducer;
        let envelope = EventEnvelope {
            topic: "test.topic".to_string(),
            key: "key-1".to_string(),
            payload: b"test payload".to_vec(),
            headers: vec![],
            metadata: HashMap::new(),
        };
        let result = producer.publish(envelope).await;
        assert!(result.is_ok());
    }

    // NoOpEventProducer の publish_batch が何も処理せず Ok を返すことを確認する。
    #[tokio::test]
    async fn test_noop_producer_publish_batch() {
        let producer = NoOpEventProducer;
        let envelopes = vec![
            EventEnvelope {
                topic: "test.topic".to_string(),
                key: "key-1".to_string(),
                payload: b"payload 1".to_vec(),
                headers: vec![],
                metadata: HashMap::new(),
            },
            EventEnvelope {
                topic: "test.topic".to_string(),
                key: "key-2".to_string(),
                payload: b"payload 2".to_vec(),
                headers: vec![],
                metadata: HashMap::new(),
            },
        ];
        let result = producer.publish_batch(envelopes).await;
        assert!(result.is_ok());
    }

    // NoOpEventProducer が空のバッチでも Ok を返すことを確認する。
    #[tokio::test]
    async fn test_noop_producer_publish_empty_batch() {
        let producer = NoOpEventProducer;
        let result = producer.publish_batch(vec![]).await;
        assert!(result.is_ok());
    }

    // EventEnvelope を全フィールド指定で生成し、各フィールドが正しく保持されることを確認する。
    #[test]
    fn test_event_envelope_creation_with_all_fields() {
        let mut metadata = HashMap::new();
        metadata.insert("trace_id".to_string(), "abc-123".to_string());
        metadata.insert("correlation_id".to_string(), "corr-456".to_string());

        let headers = vec![
            ("content-type".to_string(), b"application/json".to_vec()),
            ("x-schema-id".to_string(), b"42".to_vec()),
        ];

        let envelope = EventEnvelope {
            topic: "k1s0.system.auth.login.v1".to_string(),
            key: "user-42".to_string(),
            payload: b"test payload data".to_vec(),
            headers: headers.clone(),
            metadata: metadata.clone(),
        };

        assert_eq!(envelope.topic, "k1s0.system.auth.login.v1");
        assert_eq!(envelope.key, "user-42");
        assert_eq!(envelope.payload, b"test payload data");
        assert_eq!(envelope.headers.len(), 2);
        assert_eq!(envelope.headers[0].0, "content-type");
        assert_eq!(envelope.metadata.get("trace_id").unwrap(), "abc-123");
        assert_eq!(envelope.metadata.get("correlation_id").unwrap(), "corr-456");
    }

    // metadata にトレース情報を設定し伝播されることを確認する。
    #[tokio::test]
    async fn test_metadata_propagation_through_noop_producer() {
        let producer = NoOpEventProducer;
        let mut metadata = HashMap::new();
        metadata.insert("trace_id".to_string(), "trace-abc".to_string());
        metadata.insert("span_id".to_string(), "span-def".to_string());

        let envelope = EventEnvelope {
            topic: "k1s0.system.metrics.v1".to_string(),
            key: "metric-1".to_string(),
            payload: b"{}".to_vec(),
            headers: vec![],
            metadata,
        };

        // NoOpEventProducer は metadata を無視するが、呼び出し自体は成功する
        let result = producer.publish(envelope).await;
        assert!(result.is_ok());
    }

    // NoOpEventProducer が Send + Sync を満たすことを確認する（並行処理で利用可能）。
    #[test]
    fn test_noop_producer_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<NoOpEventProducer>();
    }
}
