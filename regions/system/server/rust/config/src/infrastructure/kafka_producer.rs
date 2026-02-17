use async_trait::async_trait;
use serde::Deserialize;

use crate::domain::entity::config_change_log::ConfigChangeLog;

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

/// ConfigChangeEventPublisher は設定変更イベント配信のためのトレイト。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ConfigChangeEventPublisher: Send + Sync {
    async fn publish(&self, event: &ConfigChangeLog) -> anyhow::Result<()>;
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
            .unwrap_or_else(|| "k1s0.system.config.changed.v1".to_string());

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

    /// 配信先トピック名を返す。
    pub fn topic(&self) -> &str {
        &self.topic
    }
}

#[async_trait]
impl ConfigChangeEventPublisher for KafkaProducer {
    async fn publish(&self, event: &ConfigChangeLog) -> anyhow::Result<()> {
        use rdkafka::producer::FutureRecord;
        use std::time::Duration;

        let payload = serde_json::to_vec(event)?;
        let key = format!("{}/{}", event.namespace, event.key);

        let record = FutureRecord::to(&self.topic)
            .key(&key)
            .payload(&payload);

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(err, _)| anyhow::anyhow!("failed to publish config change event: {}", err))?;

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
    use crate::domain::entity::config_change_log::{ConfigChangeLog, CreateChangeLogRequest};
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
    impl ConfigChangeEventPublisher for InMemoryProducer {
        async fn publish(&self, event: &ConfigChangeLog) -> anyhow::Result<()> {
            if self.should_fail {
                return Err(anyhow::anyhow!("broker connection refused"));
            }
            let payload = serde_json::to_vec(event)?;
            let key = format!("{}/{}", event.namespace, event.key);
            self.messages.lock().unwrap().push((key, payload));
            Ok(())
        }

        async fn close(&self) -> anyhow::Result<()> {
            Ok(())
        }
    }

    fn make_test_change_log() -> ConfigChangeLog {
        ConfigChangeLog::new(CreateChangeLogRequest {
            config_entry_id: Uuid::new_v4(),
            namespace: "system.auth.database".to_string(),
            key: "max_connections".to_string(),
            old_value: Some(serde_json::json!(25)),
            new_value: Some(serde_json::json!(50)),
            old_version: 3,
            new_version: 4,
            change_type: "UPDATED".to_string(),
            changed_by: "operator@example.com".to_string(),
        })
    }

    #[test]
    fn test_kafka_config_deserialization() {
        let yaml = r#"
brokers:
  - "kafka-0.messaging.svc.cluster.local:9092"
  - "kafka-1.messaging.svc.cluster.local:9092"
consumer_group: "config-server.default"
security_protocol: "PLAINTEXT"
sasl:
  mechanism: "SCRAM-SHA-512"
  username: "user"
  password: "pass"
topics:
  publish:
    - "k1s0.system.config.changed.v1"
  subscribe: []
"#;
        let config: KafkaConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.brokers.len(), 2);
        assert_eq!(config.consumer_group, "config-server.default");
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
        let log = make_test_change_log();

        let result = producer.publish(&log).await;
        assert!(result.is_ok());

        let messages = producer.messages.lock().unwrap();
        assert_eq!(messages.len(), 1);

        // JSON に正常変換されていることを確認
        let deserialized: ConfigChangeLog = serde_json::from_slice(&messages[0].1).unwrap();
        assert_eq!(deserialized.namespace, "system.auth.database");
        assert_eq!(deserialized.key, "max_connections");
        assert_eq!(deserialized.change_type, "UPDATED");
        assert_eq!(deserialized.old_value, Some(serde_json::json!(25)));
        assert_eq!(deserialized.new_value, Some(serde_json::json!(50)));
    }

    #[tokio::test]
    async fn test_publish_key_is_namespace_key() {
        let producer = InMemoryProducer::new();
        let log = make_test_change_log();

        producer.publish(&log).await.unwrap();

        let messages = producer.messages.lock().unwrap();
        assert_eq!(messages.len(), 1);
        // パーティションキーが namespace/key であることを確認
        assert_eq!(messages[0].0, "system.auth.database/max_connections");
    }

    #[tokio::test]
    async fn test_publish_connection_error() {
        let producer = InMemoryProducer::with_error();
        let log = make_test_change_log();

        let result = producer.publish(&log).await;
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
            .unwrap_or_else(|| "k1s0.system.config.changed.v1".to_string());
        assert_eq!(topic, "k1s0.system.config.changed.v1");
    }

    #[test]
    fn test_configured_topic_name() {
        let yaml = r#"
brokers:
  - "localhost:9092"
topics:
  publish:
    - "k1s0.system.config.changed.v1"
  subscribe: []
"#;
        let config: KafkaConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.topics.publish[0], "k1s0.system.config.changed.v1");
    }

    #[tokio::test]
    async fn test_mock_config_change_event_publisher() {
        let mut mock = MockConfigChangeEventPublisher::new();
        mock.expect_publish().returning(|_| Ok(()));
        mock.expect_close().returning(|| Ok(()));

        let log = make_test_change_log();
        assert!(mock.publish(&log).await.is_ok());
        assert!(mock.close().await.is_ok());
    }
}
