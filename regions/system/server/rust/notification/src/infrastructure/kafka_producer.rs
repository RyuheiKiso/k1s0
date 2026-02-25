use async_trait::async_trait;

use crate::domain::entity::notification_log::NotificationLog;
use crate::infrastructure::config::KafkaConfig;

/// NotificationSentEvent は通知配信完了時に Kafka へ発行するイベント。
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct NotificationSentEvent {
    pub notification_id: String,
    pub channel_id: String,
    pub recipient: String,
    pub status: String,
    pub timestamp: String, // ISO 8601
}

/// NotificationEventPublisher は通知配信イベントの Kafka 発行トレイト。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait NotificationEventPublisher: Send + Sync {
    async fn publish_notification_sent(&self, log: &NotificationLog) -> anyhow::Result<()>;
    async fn close(&self) -> anyhow::Result<()>;
}

/// NoopNotificationEventPublisher は何もしないダミー実装。
/// Kafka 未設定時のフォールバックに使用する。
pub struct NoopNotificationEventPublisher;

#[async_trait]
impl NotificationEventPublisher for NoopNotificationEventPublisher {
    async fn publish_notification_sent(&self, _log: &NotificationLog) -> anyhow::Result<()> {
        Ok(())
    }

    async fn close(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

/// KafkaNotificationProducer は rdkafka FutureProducer を使った Kafka プロデューサー。
pub struct KafkaNotificationProducer {
    producer: rdkafka::producer::FutureProducer,
    topic: String,
    metrics: Option<std::sync::Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl KafkaNotificationProducer {
    /// 新しい KafkaNotificationProducer を作成する。
    pub fn new(config: &KafkaConfig) -> anyhow::Result<Self> {
        use rdkafka::config::ClientConfig;

        let topic = "k1s0.system.notification.sent.v1".to_string();

        let mut client_config = ClientConfig::new();
        client_config.set("bootstrap.servers", config.brokers.join(","));
        client_config.set("security.protocol", &config.security_protocol);
        client_config.set("acks", "all");
        client_config.set("message.timeout.ms", "5000");

        let producer: rdkafka::producer::FutureProducer = client_config.create()?;

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
impl NotificationEventPublisher for KafkaNotificationProducer {
    async fn publish_notification_sent(&self, log: &NotificationLog) -> anyhow::Result<()> {
        use rdkafka::producer::FutureRecord;
        use std::time::Duration;

        let event = NotificationSentEvent {
            notification_id: log.id.to_string(),
            channel_id: log.channel_id.to_string(),
            recipient: log.recipient.clone(),
            status: log.status.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let payload = serde_json::to_vec(&event)?;
        let key = log.id.to_string();

        let record = FutureRecord::to(&self.topic).key(&key).payload(&payload);

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(err, _)| {
                anyhow::anyhow!("failed to publish notification sent event: {}", err)
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
    use crate::domain::entity::notification_log::NotificationLog;
    use std::sync::Mutex;
    use uuid::Uuid;

    /// テスト用のインメモリプロデューサー。
    struct InMemoryProducer {
        messages: Mutex<Vec<(String, Vec<u8>)>>,
        should_fail: bool,
    }

    impl InMemoryProducer {
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
    impl NotificationEventPublisher for InMemoryProducer {
        async fn publish_notification_sent(&self, log: &NotificationLog) -> anyhow::Result<()> {
            if self.should_fail {
                return Err(anyhow::anyhow!("broker connection refused"));
            }
            let event = NotificationSentEvent {
                notification_id: log.id.to_string(),
                channel_id: log.channel_id.to_string(),
                recipient: log.recipient.clone(),
                status: log.status.clone(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            let payload = serde_json::to_vec(&event)?;
            let key = log.id.to_string();
            self.messages.lock().unwrap().push((key, payload));
            Ok(())
        }

        async fn close(&self) -> anyhow::Result<()> {
            Ok(())
        }
    }

    fn make_test_log() -> NotificationLog {
        NotificationLog {
            id: Uuid::new_v4(),
            channel_id: Uuid::new_v4(),
            template_id: None,
            recipient: "user@example.com".to_string(),
            subject: Some("Test Subject".to_string()),
            body: "Test body".to_string(),
            status: "sent".to_string(),
            error_message: None,
            sent_at: Some(chrono::Utc::now()),
            created_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_noop_publisher() {
        let noop = NoopNotificationEventPublisher;
        let log = make_test_log();
        assert!(noop.publish_notification_sent(&log).await.is_ok());
        assert!(noop.close().await.is_ok());
    }

    #[tokio::test]
    async fn test_publish_serialization() {
        let producer = InMemoryProducer::new();
        let log = make_test_log();

        let result = producer.publish_notification_sent(&log).await;
        assert!(result.is_ok());

        let messages = producer.messages.lock().unwrap();
        assert_eq!(messages.len(), 1);

        // JSON に正常変換されていることを確認
        let deserialized: NotificationSentEvent =
            serde_json::from_slice(&messages[0].1).unwrap();
        assert_eq!(deserialized.recipient, "user@example.com");
        assert_eq!(deserialized.status, "sent");
    }

    #[tokio::test]
    async fn test_publish_key_is_notification_id() {
        let producer = InMemoryProducer::new();
        let log = make_test_log();
        let expected_key = log.id.to_string();

        producer.publish_notification_sent(&log).await.unwrap();

        let messages = producer.messages.lock().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].0, expected_key);
    }

    #[tokio::test]
    async fn test_publish_connection_error() {
        let producer = InMemoryProducer::with_error();
        let log = make_test_log();

        let result = producer.publish_notification_sent(&log).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("broker connection refused"));
    }

    #[tokio::test]
    async fn test_close_graceful() {
        let producer = InMemoryProducer::new();
        let result = producer.close().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_notification_event_publisher() {
        let mut mock = MockNotificationEventPublisher::new();
        mock.expect_publish_notification_sent()
            .returning(|_| Ok(()));
        mock.expect_close().returning(|| Ok(()));

        let log = make_test_log();
        assert!(mock.publish_notification_sent(&log).await.is_ok());
        assert!(mock.close().await.is_ok());
    }

    #[test]
    fn test_notification_sent_event_serialization() {
        let event = NotificationSentEvent {
            notification_id: Uuid::new_v4().to_string(),
            channel_id: Uuid::new_v4().to_string(),
            recipient: "user@example.com".to_string(),
            status: "sent".to_string(),
            timestamp: "2026-02-25T00:00:00Z".to_string(),
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["recipient"], "user@example.com");
        assert_eq!(json["status"], "sent");
        assert_eq!(json["timestamp"], "2026-02-25T00:00:00Z");
    }

    #[test]
    fn test_notification_sent_event_debug_format() {
        let event = NotificationSentEvent {
            notification_id: Uuid::new_v4().to_string(),
            channel_id: Uuid::new_v4().to_string(),
            recipient: "user@example.com".to_string(),
            status: "sent".to_string(),
            timestamp: "2026-02-25T00:00:00Z".to_string(),
        };
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("NotificationSentEvent"));
        assert!(debug_str.contains("user@example.com"));
    }
}
