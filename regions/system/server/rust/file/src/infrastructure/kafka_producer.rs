use async_trait::async_trait;

use crate::infrastructure::config::KafkaConfig;

/// FileEventPublisher はファイルイベントを Kafka に送信するトレイト。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait FileEventPublisher: Send + Sync {
    async fn publish(&self, event_type: &str, payload: &serde_json::Value) -> anyhow::Result<()>;
    // シャットダウン時に未送信メッセージをフラッシュして失われるのを防ぐ（AVAIL-005 監査対応）
    #[allow(dead_code)]
    async fn close(&self) -> anyhow::Result<()>;
}

/// NoopFileEventPublisher は何もしないダミー実装。
pub struct NoopFileEventPublisher;

#[async_trait]
impl FileEventPublisher for NoopFileEventPublisher {
    async fn publish(&self, _event_type: &str, _payload: &serde_json::Value) -> anyhow::Result<()> {
        Ok(())
    }

    async fn close(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

/// FileKafkaProducer は rdkafka FutureProducer を使った Kafka プロデューサー。
pub struct FileKafkaProducer {
    producer: rdkafka::producer::FutureProducer,
    topic: String,
    metrics: Option<std::sync::Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl FileKafkaProducer {
    pub fn new(config: &KafkaConfig) -> anyhow::Result<Self> {
        use rdkafka::config::ClientConfig;

        let mut client_config = ClientConfig::new();
        client_config.set("bootstrap.servers", config.brokers.join(","));
        client_config.set("security.protocol", &config.security_protocol);
        client_config.set("acks", "all");
        client_config.set("message.timeout.ms", "5000");
        // 冪等プロデューサーを有効化し、メッセージの重複送信を防止する
        client_config.set("enable.idempotence", "true");

        let producer: rdkafka::producer::FutureProducer = client_config.create()?;

        Ok(Self {
            producer,
            topic: config.topic_events.clone(),
            metrics: None,
        })
    }
}

#[async_trait]
impl FileEventPublisher for FileKafkaProducer {
    async fn publish(&self, event_type: &str, payload: &serde_json::Value) -> anyhow::Result<()> {
        use rdkafka::producer::FutureRecord;
        use std::time::Duration;

        let mut message = payload.clone();
        if let Some(map) = message.as_object_mut() {
            map.insert(
                "event_type".to_string(),
                serde_json::Value::String(event_type.to_string()),
            );
            map.insert(
                "timestamp".to_string(),
                serde_json::Value::String(chrono::Utc::now().to_rfc3339()),
            );
        }

        let payload_bytes = serde_json::to_vec(&message)?;
        let key = payload
            .get("file_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let record = FutureRecord::to(&self.topic)
            .key(&key)
            .payload(&payload_bytes);

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(err, _)| {
                anyhow::anyhow!("failed to publish file event to {}: {}", self.topic, err)
            })?;

        if let Some(ref m) = self.metrics {
            m.record_kafka_message_produced(&self.topic);
        }

        Ok(())
    }

    async fn close(&self) -> anyhow::Result<()> {
        use rdkafka::producer::Producer;
        // シャットダウン時に未送信メッセージをフラッシュして失われるのを防ぐ（AVAIL-005 監査対応）
        self.producer.flush(std::time::Duration::from_secs(10))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_noop_publisher() {
        let publisher = NoopFileEventPublisher;
        let payload = serde_json::json!({"file_id": "file_001"});
        assert!(publisher.publish("file.uploaded", &payload).await.is_ok());
        assert!(publisher.close().await.is_ok());
    }

    #[tokio::test]
    async fn test_mock_publisher() {
        let mut mock = MockFileEventPublisher::new();
        mock.expect_publish().returning(|_, _| Ok(()));
        mock.expect_close().returning(|| Ok(()));

        let payload = serde_json::json!({"file_id": "file_001"});
        assert!(mock.publish("file.uploaded", &payload).await.is_ok());
        assert!(mock.close().await.is_ok());
    }

    #[tokio::test]
    async fn test_mock_publisher_error() {
        let mut mock = MockFileEventPublisher::new();
        mock.expect_publish()
            .returning(|_, _| Err(anyhow::anyhow!("broker unavailable")));

        let payload = serde_json::json!({});
        let result = mock.publish("file.uploaded", &payload).await;
        assert!(result.is_err());
    }
}
