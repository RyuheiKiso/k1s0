use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// VaultAccessEvent は vault アクセス時に Kafka へ発行するイベント。
#[derive(Debug, Serialize, Deserialize)]
pub struct VaultAccessEvent {
    pub key_path: String,
    pub action: String,
    pub actor_id: String,
    pub success: bool,
    pub error_msg: Option<String>,
    pub timestamp: String, // ISO 8601
}

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

/// VaultEventPublisher は vault アクセスイベント配信のためのトレイト。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait VaultEventPublisher: Send + Sync {
    async fn publish_secret_accessed(&self, event: &VaultAccessEvent) -> anyhow::Result<()>;
    async fn close(&self) -> anyhow::Result<()>;
}

/// NoopVaultEventPublisher はイベントを破棄する実装（開発/テスト用）。
pub struct NoopVaultEventPublisher;

#[async_trait]
impl VaultEventPublisher for NoopVaultEventPublisher {
    async fn publish_secret_accessed(&self, _event: &VaultAccessEvent) -> anyhow::Result<()> {
        Ok(())
    }

    async fn close(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

/// KafkaProducer は rdkafka FutureProducer を使った Kafka プロデューサー。
pub struct KafkaProducer {
    producer: rdkafka::producer::FutureProducer,
    topic: String,
    metrics: Option<std::sync::Arc<k1s0_telemetry::metrics::Metrics>>,
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
            .unwrap_or_else(|| "k1s0.system.vault.access.v1".to_string());

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
impl VaultEventPublisher for KafkaProducer {
    async fn publish_secret_accessed(&self, event: &VaultAccessEvent) -> anyhow::Result<()> {
        use rdkafka::producer::FutureRecord;
        use std::time::Duration;

        let payload = serde_json::to_vec(event)?;
        let key = format!("{}:{}", event.key_path, event.action);

        let record = FutureRecord::to(&self.topic).key(&key).payload(&payload);

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(err, _)| {
                anyhow::anyhow!("failed to publish vault access event: {}", err)
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
    impl VaultEventPublisher for InMemoryProducer {
        async fn publish_secret_accessed(&self, event: &VaultAccessEvent) -> anyhow::Result<()> {
            if self.should_fail {
                return Err(anyhow::anyhow!("broker connection refused"));
            }
            let payload = serde_json::to_vec(event)?;
            let key = format!("{}:{}", event.key_path, event.action);
            self.messages.lock().unwrap().push((key, payload));
            Ok(())
        }

        async fn close(&self) -> anyhow::Result<()> {
            Ok(())
        }
    }

    fn make_test_event() -> VaultAccessEvent {
        VaultAccessEvent {
            key_path: "app/db/password".to_string(),
            action: "read".to_string(),
            actor_id: "user-1".to_string(),
            success: true,
            error_msg: None,
            timestamp: "2026-02-26T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_kafka_config_deserialization() {
        let yaml = r#"
brokers:
  - "kafka-0.messaging.svc.cluster.local:9092"
  - "kafka-1.messaging.svc.cluster.local:9092"
consumer_group: "vault-server.default"
security_protocol: "PLAINTEXT"
sasl:
  mechanism: "SCRAM-SHA-512"
  username: "user"
  password: "pass"
topics:
  publish:
    - "k1s0.system.vault.access.v1"
  subscribe: []
"#;
        let config: KafkaConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.brokers.len(), 2);
        assert_eq!(config.consumer_group, "vault-server.default");
        assert_eq!(config.sasl.mechanism, "SCRAM-SHA-512");
        assert_eq!(config.topics.publish.len(), 1);
        assert!(config.topics.subscribe.is_empty());
    }

    #[test]
    fn test_kafka_config_defaults() {
        let yaml = r#"
brokers:
  - "localhost:9092"
"#;
        let config: KafkaConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.security_protocol, "PLAINTEXT");
        assert!(config.consumer_group.is_empty());
        assert!(config.topics.publish.is_empty());
    }

    #[tokio::test]
    async fn test_publish_serialization() {
        let producer = InMemoryProducer::new();
        let event = make_test_event();

        let result = producer.publish_secret_accessed(&event).await;
        assert!(result.is_ok());

        let messages = producer.messages.lock().unwrap();
        assert_eq!(messages.len(), 1);

        let deserialized: VaultAccessEvent = serde_json::from_slice(&messages[0].1).unwrap();
        assert_eq!(deserialized.key_path, "app/db/password");
        assert_eq!(deserialized.action, "read");
        assert_eq!(deserialized.actor_id, "user-1");
        assert!(deserialized.success);
    }

    #[tokio::test]
    async fn test_publish_key_format() {
        let producer = InMemoryProducer::new();
        let event = make_test_event();

        producer.publish_secret_accessed(&event).await.unwrap();

        let messages = producer.messages.lock().unwrap();
        assert_eq!(messages[0].0, "app/db/password:read");
    }

    #[tokio::test]
    async fn test_publish_connection_error() {
        let producer = InMemoryProducer::with_error();
        let event = make_test_event();

        let result = producer.publish_secret_accessed(&event).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("broker connection refused"));
    }

    #[tokio::test]
    async fn test_noop_publisher() {
        let publisher = NoopVaultEventPublisher;
        let event = make_test_event();

        assert!(publisher.publish_secret_accessed(&event).await.is_ok());
        assert!(publisher.close().await.is_ok());
    }

    #[test]
    fn test_default_topic_name() {
        let yaml = r#"
brokers:
  - "localhost:9092"
"#;
        let config: KafkaConfig = serde_yaml::from_str(yaml).unwrap();
        let topic = config
            .topics
            .publish
            .first()
            .cloned()
            .unwrap_or_else(|| "k1s0.system.vault.access.v1".to_string());
        assert_eq!(topic, "k1s0.system.vault.access.v1");
    }

    #[test]
    fn test_vault_access_event_serialization() {
        let event = make_test_event();
        let json = serde_json::to_value(&event).unwrap();

        assert_eq!(json["key_path"], "app/db/password");
        assert_eq!(json["action"], "read");
        assert_eq!(json["actor_id"], "user-1");
        assert_eq!(json["success"], true);
        assert!(json["error_msg"].is_null());
        assert_eq!(json["timestamp"], "2026-02-26T00:00:00Z");
    }

    #[test]
    fn test_vault_access_event_with_error() {
        let event = VaultAccessEvent {
            key_path: "app/db/password".to_string(),
            action: "read".to_string(),
            actor_id: "user-1".to_string(),
            success: false,
            error_msg: Some("secret not found".to_string()),
            timestamp: "2026-02-26T00:00:00Z".to_string(),
        };

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["error_msg"], "secret not found");
    }

    #[tokio::test]
    async fn test_mock_vault_event_publisher() {
        let mut mock = MockVaultEventPublisher::new();
        mock.expect_publish_secret_accessed()
            .returning(|_| Ok(()));
        mock.expect_close().returning(|| Ok(()));

        let event = make_test_event();
        assert!(mock.publish_secret_accessed(&event).await.is_ok());
        assert!(mock.close().await.is_ok());
    }
}
