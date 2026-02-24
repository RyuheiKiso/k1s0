use async_trait::async_trait;

use crate::domain::entity::event::StoredEvent;
use crate::infrastructure::config::KafkaConfig;

/// EventPublisher はイベントストアからの Kafka イベント発行用トレイト。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait EventPublisher: Send + Sync {
    /// イベントを Kafka トピックに発行する。
    async fn publish_events(
        &self,
        stream_id: &str,
        events: &[StoredEvent],
    ) -> anyhow::Result<()>;
}

/// EventStoreKafkaProducer は rdkafka FutureProducer を使った Kafka プロデューサー。
pub struct EventStoreKafkaProducer {
    producer: rdkafka::producer::FutureProducer,
    topic: String,
    metrics: Option<std::sync::Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl EventStoreKafkaProducer {
    /// 新しい EventStoreKafkaProducer を作成する。
    pub fn new(config: &KafkaConfig) -> anyhow::Result<Self> {
        use rdkafka::config::ClientConfig;

        let mut client_config = ClientConfig::new();
        client_config.set("bootstrap.servers", config.brokers.join(","));
        client_config.set("security.protocol", &config.security_protocol);
        client_config.set("acks", &config.producer_acks);
        client_config.set(
            "message.send.max.retries",
            config.producer_retries.to_string(),
        );
        client_config.set("message.timeout.ms", "5000");

        let producer: rdkafka::producer::FutureProducer = client_config.create()?;

        Ok(Self {
            producer,
            topic: config.topic_published.clone(),
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
impl EventPublisher for EventStoreKafkaProducer {
    async fn publish_events(
        &self,
        stream_id: &str,
        events: &[StoredEvent],
    ) -> anyhow::Result<()> {
        use rdkafka::producer::FutureRecord;
        use std::time::Duration;

        for event in events {
            let payload = serde_json::to_vec(event)?;
            let key = format!("{}:{}", stream_id, event.sequence);

            let record = FutureRecord::to(&self.topic)
                .key(&key)
                .payload(&payload);

            self.producer
                .send(record, Duration::from_secs(5))
                .await
                .map_err(|(err, _)| {
                    anyhow::anyhow!(
                        "failed to publish event to topic {}: {}",
                        self.topic,
                        err
                    )
                })?;

            if let Some(ref m) = self.metrics {
                m.record_kafka_message_produced(&self.topic);
            }
        }

        Ok(())
    }
}

/// NoopEventPublisher は何もしないダミープロデューサー（Kafka無効時に使う）。
pub struct NoopEventPublisher;

#[async_trait]
impl EventPublisher for NoopEventPublisher {
    async fn publish_events(
        &self,
        _stream_id: &str,
        _events: &[StoredEvent],
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_noop_event_publisher() {
        let publisher = NoopEventPublisher;
        let result = publisher.publish_events("test-stream", &[]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_event_publisher() {
        let mut mock = MockEventPublisher::new();
        mock.expect_publish_events().returning(|_, _| Ok(()));

        let result = mock.publish_events("test-stream", &[]).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_event_publisher_error() {
        let mut mock = MockEventPublisher::new();
        mock.expect_publish_events()
            .returning(|_, _| Err(anyhow::anyhow!("broker unavailable")));

        let result = mock.publish_events("test-stream", &[]).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("broker unavailable"));
    }
}
