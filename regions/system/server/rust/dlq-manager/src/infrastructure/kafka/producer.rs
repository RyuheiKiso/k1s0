use async_trait::async_trait;

use super::KafkaConfig;

/// DlqEventPublisher は元トピックへのメッセージ再発行用トレイト。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait DlqEventPublisher: Send + Sync {
    /// 指定トピックにメッセージを再発行する。
    async fn publish_to_topic(
        &self,
        topic: &str,
        payload: &serde_json::Value,
    ) -> anyhow::Result<()>;
}

/// DlqKafkaProducer は rdkafka FutureProducer を使った Kafka プロデューサー。
pub struct DlqKafkaProducer {
    producer: rdkafka::producer::FutureProducer,
    metrics: Option<std::sync::Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl DlqKafkaProducer {
    /// 新しい DlqKafkaProducer を作成する。
    pub fn new(config: &KafkaConfig) -> anyhow::Result<Self> {
        use rdkafka::config::ClientConfig;

        let mut client_config = ClientConfig::new();
        client_config.set("bootstrap.servers", config.brokers.join(","));
        client_config.set("security.protocol", &config.security_protocol);
        client_config.set("acks", "all");
        client_config.set("message.timeout.ms", "5000");

        let producer: rdkafka::producer::FutureProducer = client_config.create()?;

        Ok(Self {
            producer,
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
impl DlqEventPublisher for DlqKafkaProducer {
    async fn publish_to_topic(
        &self,
        topic: &str,
        payload: &serde_json::Value,
    ) -> anyhow::Result<()> {
        use rdkafka::producer::FutureRecord;
        use std::time::Duration;

        let payload_bytes = serde_json::to_vec(payload)?;
        let key = uuid::Uuid::new_v4().to_string();

        let record = FutureRecord::to(topic).key(&key).payload(&payload_bytes);

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(err, _)| anyhow::anyhow!("failed to publish to topic {}: {}", topic, err))?;

        if let Some(ref m) = self.metrics {
            m.record_kafka_message_produced(topic);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_dlq_event_publisher() {
        let mut mock = MockDlqEventPublisher::new();
        mock.expect_publish_to_topic().returning(|_, _| Ok(()));

        let payload = serde_json::json!({"order_id": "123"});
        assert!(mock
            .publish_to_topic("orders.events.v1", &payload)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_mock_dlq_event_publisher_error() {
        let mut mock = MockDlqEventPublisher::new();
        mock.expect_publish_to_topic()
            .returning(|_, _| Err(anyhow::anyhow!("broker unavailable")));

        let payload = serde_json::json!({});
        let result = mock.publish_to_topic("orders.events.v1", &payload).await;
        assert!(result.is_err());
    }
}
