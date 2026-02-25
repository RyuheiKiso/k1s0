use async_trait::async_trait;
use rdkafka::producer::FutureProducer;

use crate::infrastructure::config::KafkaConfig;

/// FlagChangedEvent はフィーチャーフラグ変更時に Kafka へ発行するイベント。
#[derive(Debug, serde::Serialize)]
pub struct FlagChangedEvent {
    pub flag_key: String,
    pub enabled: bool,
    pub timestamp: String, // ISO 8601
}

/// FlagEventPublisher はフラグ変更イベント配信のためのトレイト。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait FlagEventPublisher: Send + Sync {
    async fn publish_flag_changed(&self, flag_key: &str, enabled: bool) -> anyhow::Result<()>;
    async fn close(&self) -> anyhow::Result<()>;
}

/// NoopFlagEventPublisher はイベントを発行しないスタブ実装。
pub struct NoopFlagEventPublisher;

#[async_trait]
impl FlagEventPublisher for NoopFlagEventPublisher {
    async fn publish_flag_changed(&self, _flag_key: &str, _enabled: bool) -> anyhow::Result<()> {
        Ok(())
    }

    async fn close(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

/// KafkaFlagProducer は rdkafka FutureProducer を使った Kafka プロデューサー。
pub struct KafkaFlagProducer {
    producer: FutureProducer,
    topic: String,
    metrics: Option<std::sync::Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl KafkaFlagProducer {
    /// 新しい KafkaFlagProducer を作成する。
    pub fn new(config: &KafkaConfig) -> anyhow::Result<Self> {
        use rdkafka::config::ClientConfig;

        let topic = config.topic.clone();

        let mut client_config = ClientConfig::new();
        client_config.set("bootstrap.servers", config.brokers.join(","));
        client_config.set("security.protocol", &config.security_protocol);
        client_config.set("acks", "all");
        client_config.set("message.timeout.ms", "5000");

        let producer: FutureProducer = client_config.create()?;

        Ok(Self {
            producer,
            topic,
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

    /// 配信先トピック名を返す。
    pub fn topic(&self) -> &str {
        &self.topic
    }
}

#[async_trait]
impl FlagEventPublisher for KafkaFlagProducer {
    async fn publish_flag_changed(&self, flag_key: &str, enabled: bool) -> anyhow::Result<()> {
        use rdkafka::producer::FutureRecord;
        use std::time::Duration;

        let event = FlagChangedEvent {
            flag_key: flag_key.to_string(),
            enabled,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let payload = serde_json::to_vec(&event)?;

        let record = FutureRecord::to(&self.topic)
            .key(flag_key)
            .payload(&payload);

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(err, _)| {
                anyhow::anyhow!("failed to publish flag changed event: {}", err)
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
    struct InMemoryFlagProducer {
        messages: Mutex<Vec<(String, bool)>>,
        should_fail: bool,
    }

    impl InMemoryFlagProducer {
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
    impl FlagEventPublisher for InMemoryFlagProducer {
        async fn publish_flag_changed(
            &self,
            flag_key: &str,
            enabled: bool,
        ) -> anyhow::Result<()> {
            if self.should_fail {
                return Err(anyhow::anyhow!("broker connection refused"));
            }
            self.messages
                .lock()
                .unwrap()
                .push((flag_key.to_string(), enabled));
            Ok(())
        }

        async fn close(&self) -> anyhow::Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_publish_flag_changed() {
        let producer = InMemoryFlagProducer::new();

        let result = producer
            .publish_flag_changed("feature.dark-mode", true)
            .await;
        assert!(result.is_ok());

        let messages = producer.messages.lock().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].0, "feature.dark-mode");
        assert!(messages[0].1);
    }

    #[tokio::test]
    async fn test_publish_multiple_events() {
        let producer = InMemoryFlagProducer::new();

        producer
            .publish_flag_changed("feature.dark-mode", true)
            .await
            .unwrap();
        producer
            .publish_flag_changed("feature.new-ui", false)
            .await
            .unwrap();

        let messages = producer.messages.lock().unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].0, "feature.dark-mode");
        assert_eq!(messages[1].0, "feature.new-ui");
    }

    #[tokio::test]
    async fn test_publish_connection_error() {
        let producer = InMemoryFlagProducer::with_error();

        let result = producer
            .publish_flag_changed("feature.dark-mode", true)
            .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("broker connection refused"));
    }

    #[tokio::test]
    async fn test_noop_publisher() {
        let publisher = NoopFlagEventPublisher;

        let result = publisher
            .publish_flag_changed("feature.dark-mode", true)
            .await;
        assert!(result.is_ok());

        let close_result = publisher.close().await;
        assert!(close_result.is_ok());
    }

    #[tokio::test]
    async fn test_close_graceful() {
        let producer = InMemoryFlagProducer::new();
        let result = producer.close().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_flag_changed_event_serialization() {
        let event = FlagChangedEvent {
            flag_key: "feature.dark-mode".to_string(),
            enabled: true,
            timestamp: "2026-02-25T00:00:00Z".to_string(),
        };

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["flag_key"], "feature.dark-mode");
        assert_eq!(json["enabled"], true);
        assert_eq!(json["timestamp"], "2026-02-25T00:00:00Z");
    }

    #[test]
    fn test_flag_changed_event_debug_format() {
        let event = FlagChangedEvent {
            flag_key: "feature.dark-mode".to_string(),
            enabled: true,
            timestamp: "2026-02-25T00:00:00Z".to_string(),
        };

        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("FlagChangedEvent"));
        assert!(debug_str.contains("feature.dark-mode"));
    }

    #[tokio::test]
    async fn test_mock_flag_event_publisher() {
        let mut mock = MockFlagEventPublisher::new();
        mock.expect_publish_flag_changed().returning(|_, _| Ok(()));
        mock.expect_close().returning(|| Ok(()));

        assert!(mock
            .publish_flag_changed("feature.dark-mode", true)
            .await
            .is_ok());
        assert!(mock.close().await.is_ok());
    }
}
