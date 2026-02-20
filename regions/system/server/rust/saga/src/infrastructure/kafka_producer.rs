use async_trait::async_trait;
use serde::Deserialize;

/// KafkaConfig は Kafka 接続の設定を表す。
#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    pub brokers: Vec<String>,
    #[serde(default)]
    pub consumer_group: String,
    #[serde(default = "default_security_protocol")]
    pub security_protocol: String,
    #[serde(default)]
    pub sasl: SaslConfig,
    #[serde(default)]
    pub topics: TopicsConfig,
}

fn default_security_protocol() -> String {
    "PLAINTEXT".to_string()
}

/// SaslConfig は SASL 認証の設定を表す。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SaslConfig {
    #[serde(default)]
    pub mechanism: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
}

/// TopicsConfig はトピック設定を表す。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TopicsConfig {
    #[serde(default)]
    pub publish: Vec<String>,
    #[serde(default)]
    pub subscribe: Vec<String>,
}

/// SagaEventPublisher はSagaイベント配信のためのトレイト。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SagaEventPublisher: Send + Sync {
    async fn publish_saga_event(
        &self,
        saga_id: &str,
        event_type: &str,
        payload: &serde_json::Value,
    ) -> anyhow::Result<()>;
    async fn close(&self) -> anyhow::Result<()>;
}

/// KafkaProducer は rdkafka FutureProducer を使った Kafka プロデューサー。
pub struct KafkaProducer {
    producer: rdkafka::producer::FutureProducer,
    topic: String,
}

impl KafkaProducer {
    /// 新しい KafkaProducer を作成する。
    pub fn new(config: &KafkaConfig) -> anyhow::Result<Self> {
        use rdkafka::config::ClientConfig;

        let topic = config
            .topics
            .publish
            .first()
            .cloned()
            .unwrap_or_else(|| "k1s0.system.saga.events.v1".to_string());

        let mut client_config = ClientConfig::new();
        client_config.set("bootstrap.servers", config.brokers.join(","));
        client_config.set("security.protocol", &config.security_protocol);
        client_config.set("acks", "all");
        client_config.set("message.timeout.ms", "5000");

        if !config.sasl.mechanism.is_empty() {
            client_config.set("sasl.mechanism", &config.sasl.mechanism);
            client_config.set("sasl.username", &config.sasl.username);
            client_config.set("sasl.password", &config.sasl.password);
        }

        let producer: rdkafka::producer::FutureProducer = client_config.create()?;

        Ok(Self { producer, topic })
    }
}

#[async_trait]
impl SagaEventPublisher for KafkaProducer {
    async fn publish_saga_event(
        &self,
        saga_id: &str,
        event_type: &str,
        payload: &serde_json::Value,
    ) -> anyhow::Result<()> {
        use rdkafka::producer::FutureRecord;
        use std::time::Duration;

        let event = serde_json::json!({
            "saga_id": saga_id,
            "event_type": event_type,
            "payload": payload,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        let event_bytes = serde_json::to_vec(&event)?;

        let record = FutureRecord::to(&self.topic)
            .key(saga_id)
            .payload(&event_bytes);

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(err, _)| anyhow::anyhow!("failed to publish saga event: {}", err))?;

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
    impl SagaEventPublisher for InMemoryProducer {
        async fn publish_saga_event(
            &self,
            saga_id: &str,
            event_type: &str,
            payload: &serde_json::Value,
        ) -> anyhow::Result<()> {
            if self.should_fail {
                return Err(anyhow::anyhow!("broker connection refused"));
            }
            let event = serde_json::json!({
                "saga_id": saga_id,
                "event_type": event_type,
                "payload": payload,
            });
            let bytes = serde_json::to_vec(&event)?;
            self.messages
                .lock()
                .unwrap()
                .push((saga_id.to_string(), bytes));
            Ok(())
        }

        async fn close(&self) -> anyhow::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_kafka_config_deserialization() {
        let yaml = r#"
brokers:
  - "kafka-0.messaging.svc.cluster.local:9092"
consumer_group: "saga-server.default"
security_protocol: "PLAINTEXT"
topics:
  publish:
    - "k1s0.system.saga.events.v1"
  subscribe: []
"#;
        let config: KafkaConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.brokers.len(), 1);
        assert_eq!(config.topics.publish.len(), 1);
    }

    #[test]
    fn test_kafka_config_defaults() {
        let yaml = r#"
brokers:
  - "localhost:9092"
"#;
        let config: KafkaConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.security_protocol, "PLAINTEXT");
        assert!(config.topics.publish.is_empty());
    }

    #[tokio::test]
    async fn test_publish_saga_event() {
        let producer = InMemoryProducer::new();
        let payload = serde_json::json!({"order_id": "123"});

        let result = producer
            .publish_saga_event("saga-001", "SAGA_STARTED", &payload)
            .await;
        assert!(result.is_ok());

        let messages = producer.messages.lock().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].0, "saga-001");
    }

    #[tokio::test]
    async fn test_publish_connection_error() {
        let producer = InMemoryProducer::with_error();
        let payload = serde_json::json!({});

        let result = producer
            .publish_saga_event("saga-001", "SAGA_STARTED", &payload)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_saga_event_publisher() {
        let mut mock = MockSagaEventPublisher::new();
        mock.expect_publish_saga_event().returning(|_, _, _| Ok(()));
        mock.expect_close().returning(|| Ok(()));

        let payload = serde_json::json!({});
        assert!(mock
            .publish_saga_event("saga-001", "COMPLETED", &payload)
            .await
            .is_ok());
        assert!(mock.close().await.is_ok());
    }
}
