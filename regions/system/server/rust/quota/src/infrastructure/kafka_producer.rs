use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// QuotaExceededEvent はクォータ超過時に Kafka へ発行するイベント。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaExceededEvent {
    pub event_type: String, // "QUOTA_EXCEEDED"
    pub quota_id: String,
    pub subject_type: String,
    pub subject_id: String,
    pub period: String,
    pub limit: u64,
    pub used: u64,
    pub exceeded_at: String, // ISO 8601
    pub reset_at: String,
}

/// QuotaThresholdReachedEvent はクォータ閾値到達時に Kafka へ発行するイベント。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaThresholdReachedEvent {
    pub event_type: String, // "QUOTA_THRESHOLD_REACHED"
    pub quota_id: String,
    pub subject_type: String,
    pub subject_id: String,
    pub period: String,
    pub limit: u64,
    pub used: u64,
    pub usage_percent: f64,
    pub alert_threshold_percent: u8,
    pub reached_at: String, // ISO 8601
}

/// QuotaEventPublisher はクォータイベント配信のためのトレイト。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait QuotaEventPublisher: Send + Sync {
    async fn publish_quota_exceeded(&self, event: &QuotaExceededEvent) -> anyhow::Result<()>;
    async fn publish_threshold_reached(
        &self,
        event: &QuotaThresholdReachedEvent,
    ) -> anyhow::Result<()>;
    async fn close(&self) -> anyhow::Result<()>;
}

/// NoopQuotaEventPublisher は何もしないデフォルト実装。
pub struct NoopQuotaEventPublisher;

#[async_trait]
impl QuotaEventPublisher for NoopQuotaEventPublisher {
    async fn publish_quota_exceeded(&self, _event: &QuotaExceededEvent) -> anyhow::Result<()> {
        tracing::debug!("NoopQuotaEventPublisher: quota exceeded event discarded");
        Ok(())
    }

    async fn publish_threshold_reached(
        &self,
        _event: &QuotaThresholdReachedEvent,
    ) -> anyhow::Result<()> {
        tracing::debug!("NoopQuotaEventPublisher: quota threshold reached event discarded");
        Ok(())
    }

    async fn close(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

/// KafkaQuotaProducer は rdkafka FutureProducer を使った Kafka プロデューサー。
pub struct KafkaQuotaProducer {
    producer: rdkafka::producer::FutureProducer,
    topic_exceeded: String,
    topic_threshold: String,
    metrics: Option<std::sync::Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl KafkaQuotaProducer {
    /// 新しい KafkaQuotaProducer を作成する。
    pub fn new(
        brokers: &str,
        security_protocol: &str,
        topic_exceeded: &str,
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
            topic_exceeded: topic_exceeded.to_string(),
            topic_threshold: "k1s0.system.quota.threshold.reached.v1".to_string(),
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
impl QuotaEventPublisher for KafkaQuotaProducer {
    async fn publish_quota_exceeded(&self, event: &QuotaExceededEvent) -> anyhow::Result<()> {
        use rdkafka::producer::FutureRecord;
        use std::time::Duration;

        let payload = serde_json::to_vec(event)?;
        let key = format!("{}:{}", event.subject_type, event.subject_id);

        let record = FutureRecord::to(&self.topic_exceeded)
            .key(&key)
            .payload(&payload);

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(err, _)| {
                anyhow::anyhow!("failed to publish quota exceeded event: {}", err)
            })?;

        if let Some(ref m) = self.metrics {
            m.record_kafka_message_produced(&self.topic_exceeded);
        }

        Ok(())
    }

    async fn publish_threshold_reached(
        &self,
        event: &QuotaThresholdReachedEvent,
    ) -> anyhow::Result<()> {
        use rdkafka::producer::FutureRecord;
        use std::time::Duration;

        let payload = serde_json::to_vec(event)?;
        let key = format!("{}:{}", event.subject_type, event.subject_id);

        let record = FutureRecord::to(&self.topic_threshold)
            .key(&key)
            .payload(&payload);

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(err, _)| {
                anyhow::anyhow!("failed to publish quota threshold reached event: {}", err)
            })?;

        if let Some(ref m) = self.metrics {
            m.record_kafka_message_produced(&self.topic_threshold);
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
    impl QuotaEventPublisher for InMemoryProducer {
        async fn publish_quota_exceeded(&self, event: &QuotaExceededEvent) -> anyhow::Result<()> {
            if self.should_fail {
                return Err(anyhow::anyhow!("broker connection refused"));
            }
            let payload = serde_json::to_vec(event)?;
            let key = format!("{}:{}", event.subject_type, event.subject_id);
            self.messages.lock().unwrap().push((key, payload));
            Ok(())
        }

        async fn publish_threshold_reached(
            &self,
            event: &QuotaThresholdReachedEvent,
        ) -> anyhow::Result<()> {
            if self.should_fail {
                return Err(anyhow::anyhow!("broker connection refused"));
            }
            let payload = serde_json::to_vec(event)?;
            let key = format!("{}:{}", event.subject_type, event.subject_id);
            self.messages.lock().unwrap().push((key, payload));
            Ok(())
        }

        async fn close(&self) -> anyhow::Result<()> {
            Ok(())
        }
    }

    fn make_exceeded_event() -> QuotaExceededEvent {
        QuotaExceededEvent {
            event_type: "QUOTA_EXCEEDED".to_string(),
            quota_id: "quota_abc123".to_string(),
            subject_type: "tenant".to_string(),
            subject_id: "tenant-xyz".to_string(),
            period: "daily".to_string(),
            limit: 10000,
            used: 10001,
            exceeded_at: "2026-02-25T00:00:00Z".to_string(),
            reset_at: "".to_string(),
        }
    }

    fn make_threshold_event() -> QuotaThresholdReachedEvent {
        QuotaThresholdReachedEvent {
            event_type: "QUOTA_THRESHOLD_REACHED".to_string(),
            quota_id: "quota_abc123".to_string(),
            subject_type: "tenant".to_string(),
            subject_id: "tenant-xyz".to_string(),
            period: "daily".to_string(),
            limit: 10000,
            used: 8000,
            usage_percent: 80.0,
            alert_threshold_percent: 80,
            reached_at: "2026-02-25T00:00:00Z".to_string(),
        }
    }

    #[tokio::test]
    async fn test_publish_exceeded_serialization() {
        let producer = InMemoryProducer::new();
        let event = make_exceeded_event();

        let result = producer.publish_quota_exceeded(&event).await;
        assert!(result.is_ok());

        let messages = producer.messages.lock().unwrap();
        assert_eq!(messages.len(), 1);

        let deserialized: QuotaExceededEvent = serde_json::from_slice(&messages[0].1).unwrap();
        assert_eq!(deserialized.quota_id, "quota_abc123");
        assert_eq!(deserialized.event_type, "QUOTA_EXCEEDED");
        assert_eq!(deserialized.subject_type, "tenant");
        assert_eq!(deserialized.limit, 10000);
        assert_eq!(deserialized.used, 10001);
    }

    #[tokio::test]
    async fn test_publish_threshold_serialization() {
        let producer = InMemoryProducer::new();
        let event = make_threshold_event();

        let result = producer.publish_threshold_reached(&event).await;
        assert!(result.is_ok());

        let messages = producer.messages.lock().unwrap();
        assert_eq!(messages.len(), 1);

        let deserialized: QuotaThresholdReachedEvent =
            serde_json::from_slice(&messages[0].1).unwrap();
        assert_eq!(deserialized.quota_id, "quota_abc123");
        assert_eq!(deserialized.event_type, "QUOTA_THRESHOLD_REACHED");
        assert_eq!(deserialized.alert_threshold_percent, 80);
        assert!((deserialized.usage_percent - 80.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_publish_key_format() {
        let producer = InMemoryProducer::new();
        let event = make_exceeded_event();

        producer.publish_quota_exceeded(&event).await.unwrap();

        let messages = producer.messages.lock().unwrap();
        assert_eq!(messages[0].0, "tenant:tenant-xyz");
    }

    #[tokio::test]
    async fn test_publish_connection_error() {
        let producer = InMemoryProducer::with_error();
        let event = make_exceeded_event();

        let result = producer.publish_quota_exceeded(&event).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("broker connection refused"));
    }

    #[tokio::test]
    async fn test_noop_publisher() {
        let publisher = NoopQuotaEventPublisher;
        let exceeded_event = make_exceeded_event();
        let threshold_event = make_threshold_event();

        assert!(publisher
            .publish_quota_exceeded(&exceeded_event)
            .await
            .is_ok());
        assert!(publisher
            .publish_threshold_reached(&threshold_event)
            .await
            .is_ok());
        assert!(publisher.close().await.is_ok());
    }

    #[tokio::test]
    async fn test_close_graceful() {
        let producer = InMemoryProducer::new();
        let result = producer.close().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_quota_event_publisher() {
        let mut mock = MockQuotaEventPublisher::new();
        mock.expect_publish_quota_exceeded().returning(|_| Ok(()));
        mock.expect_publish_threshold_reached()
            .returning(|_| Ok(()));
        mock.expect_close().returning(|| Ok(()));

        let exceeded_event = make_exceeded_event();
        let threshold_event = make_threshold_event();
        assert!(mock.publish_quota_exceeded(&exceeded_event).await.is_ok());
        assert!(mock
            .publish_threshold_reached(&threshold_event)
            .await
            .is_ok());
        assert!(mock.close().await.is_ok());
    }

    #[test]
    fn test_quota_exceeded_event_serialization() {
        let event = make_exceeded_event();
        let json = serde_json::to_value(&event).unwrap();

        assert_eq!(json["event_type"], "QUOTA_EXCEEDED");
        assert_eq!(json["quota_id"], "quota_abc123");
        assert_eq!(json["subject_type"], "tenant");
        assert_eq!(json["subject_id"], "tenant-xyz");
        assert_eq!(json["period"], "daily");
        assert_eq!(json["limit"], 10000);
        assert_eq!(json["used"], 10001);
        assert_eq!(json["exceeded_at"], "2026-02-25T00:00:00Z");
    }

    #[test]
    fn test_quota_threshold_event_serialization() {
        let event = make_threshold_event();
        let json = serde_json::to_value(&event).unwrap();

        assert_eq!(json["event_type"], "QUOTA_THRESHOLD_REACHED");
        assert_eq!(json["quota_id"], "quota_abc123");
        assert_eq!(json["alert_threshold_percent"], 80);
        assert_eq!(json["usage_percent"], 80.0);
    }

    #[test]
    fn test_quota_exceeded_event_debug_format() {
        let event = make_exceeded_event();
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("QuotaExceededEvent"));
        assert!(debug_str.contains("quota_abc123"));
    }
}
