use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// DocumentIndexedEvent はドキュメントインデックス登録時に Kafka へ発行するイベント。
#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentIndexedEvent {
    pub index_name: String,
    pub document_id: String,
    pub timestamp: String, // ISO 8601
}

/// SearchEventPublisher はドキュメントインデックスイベント配信のためのトレイト。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SearchEventPublisher: Send + Sync {
    async fn publish_document_indexed(&self, event: &DocumentIndexedEvent) -> anyhow::Result<()>;
    async fn close(&self) -> anyhow::Result<()>;
}

/// NoopSearchEventPublisher はイベント配信を行わない実装。
/// テストやKafka未設定環境で使用。
pub struct NoopSearchEventPublisher;

#[async_trait]
impl SearchEventPublisher for NoopSearchEventPublisher {
    async fn publish_document_indexed(&self, _event: &DocumentIndexedEvent) -> anyhow::Result<()> {
        Ok(())
    }

    async fn close(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

/// KafkaSearchProducer は rdkafka FutureProducer を使った Kafka プロデューサー。
pub struct KafkaSearchProducer {
    producer: rdkafka::producer::FutureProducer,
    topic: String,
    metrics: Option<std::sync::Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl KafkaSearchProducer {
    /// 新しい KafkaSearchProducer を作成する。
    pub fn new(
        brokers: &str,
        security_protocol: &str,
        topic: &str,
    ) -> anyhow::Result<Self> {
        use rdkafka::config::ClientConfig;

        let mut client_config = ClientConfig::new();
        client_config.set("bootstrap.servers", brokers);
        client_config.set("security.protocol", security_protocol);
        client_config.set("acks", "all");
        client_config.set("message.timeout.ms", "5000");

        let producer: rdkafka::producer::FutureProducer = client_config.create()?;

        Ok(Self {
            producer,
            topic: topic.to_string(),
            metrics: None,
        })
    }

    /// メトリクスを設定する。
    pub fn with_metrics(
        mut self,
        metrics: std::sync::Arc<k1s0_telemetry::metrics::Metrics>,
    ) -> Self {
        self.metrics = Some(metrics);
        self
    }
}

#[async_trait]
impl SearchEventPublisher for KafkaSearchProducer {
    async fn publish_document_indexed(&self, event: &DocumentIndexedEvent) -> anyhow::Result<()> {
        use rdkafka::producer::FutureRecord;
        use std::time::Duration;

        let payload = serde_json::to_vec(event)?;
        let key = format!("{}:{}", event.index_name, event.document_id);

        let record = FutureRecord::to(&self.topic).key(&key).payload(&payload);

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(err, _)| {
                anyhow::anyhow!("failed to publish document indexed event: {}", err)
            })?;

        if let Some(ref m) = self.metrics {
            m.record_kafka_message_produced(&self.topic);
        }

        Ok(())
    }

    async fn close(&self) -> anyhow::Result<()> {
        use rdkafka::producer::Producer;
        self.producer.flush(std::time::Duration::from_secs(5))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// テスト用のインメモリプロデューサー。
    struct InMemorySearchProducer {
        messages: Mutex<Vec<(String, Vec<u8>)>>,
        should_fail: bool,
    }

    impl InMemorySearchProducer {
        fn new() -> Self {
            Self {
                messages: Mutex::new(Vec::new()),
                should_fail: false,
            }
        }

        fn with_error() -> Self {
            Self {
                messages: Mutex::new(Vec::new()),
                should_fail: true,
            }
        }
    }

    #[async_trait]
    impl SearchEventPublisher for InMemorySearchProducer {
        async fn publish_document_indexed(
            &self,
            event: &DocumentIndexedEvent,
        ) -> anyhow::Result<()> {
            if self.should_fail {
                return Err(anyhow::anyhow!("broker connection refused"));
            }
            let payload = serde_json::to_vec(event)?;
            let key = format!("{}:{}", event.index_name, event.document_id);
            self.messages.lock().unwrap().push((key, payload));
            Ok(())
        }

        async fn close(&self) -> anyhow::Result<()> {
            Ok(())
        }
    }

    fn make_event() -> DocumentIndexedEvent {
        DocumentIndexedEvent {
            index_name: "products".to_string(),
            document_id: "doc-1".to_string(),
            timestamp: "2026-02-26T00:00:00Z".to_string(),
        }
    }

    #[tokio::test]
    async fn test_publish_serialization() {
        let producer = InMemorySearchProducer::new();
        let event = make_event();

        let result = producer.publish_document_indexed(&event).await;
        assert!(result.is_ok());

        let messages = producer.messages.lock().unwrap();
        assert_eq!(messages.len(), 1);

        let deserialized: DocumentIndexedEvent =
            serde_json::from_slice(&messages[0].1).unwrap();
        assert_eq!(deserialized.index_name, "products");
        assert_eq!(deserialized.document_id, "doc-1");
    }

    #[tokio::test]
    async fn test_publish_key_format() {
        let producer = InMemorySearchProducer::new();
        let event = make_event();

        producer.publish_document_indexed(&event).await.unwrap();

        let messages = producer.messages.lock().unwrap();
        assert_eq!(messages[0].0, "products:doc-1");
    }

    #[tokio::test]
    async fn test_publish_connection_error() {
        let producer = InMemorySearchProducer::with_error();
        let event = make_event();

        let result = producer.publish_document_indexed(&event).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("broker connection refused"));
    }

    #[tokio::test]
    async fn test_noop_publisher() {
        let publisher = NoopSearchEventPublisher;
        let event = make_event();

        assert!(publisher.publish_document_indexed(&event).await.is_ok());
        assert!(publisher.close().await.is_ok());
    }

    #[tokio::test]
    async fn test_close_graceful() {
        let producer = InMemorySearchProducer::new();
        let result = producer.close().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_event_serialization() {
        let event = make_event();
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["index_name"], "products");
        assert_eq!(json["document_id"], "doc-1");
        assert_eq!(json["timestamp"], "2026-02-26T00:00:00Z");
    }

    #[test]
    fn test_event_debug_format() {
        let event = make_event();
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("DocumentIndexedEvent"));
        assert!(debug_str.contains("products"));
    }

    #[tokio::test]
    async fn test_mock_search_event_publisher() {
        let mut mock = MockSearchEventPublisher::new();
        mock.expect_publish_document_indexed()
            .returning(|_| Ok(()));
        mock.expect_close().returning(|| Ok(()));

        let event = make_event();
        assert!(mock.publish_document_indexed(&event).await.is_ok());
        assert!(mock.close().await.is_ok());
    }
}
