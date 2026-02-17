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

/// KafkaProducer は Kafka へのメッセージ送信を行う（スタブ実装）。
/// 実際の rdkafka 依存はインテグレーションテスト時に使用する。
pub struct KafkaProducer {
    config: KafkaConfig,
}

impl KafkaProducer {
    pub fn new(config: KafkaConfig) -> Self {
        Self { config }
    }

    /// 監査ログイベントを Kafka に送信する。
    pub async fn send_audit_event(
        &self,
        topic: &str,
        key: &str,
        payload: &[u8],
    ) -> anyhow::Result<()> {
        tracing::info!(
            topic = topic,
            key = key,
            brokers = ?self.config.brokers,
            "sending audit event to kafka (stub)"
        );
        // 実際の rdkafka 実装はインフラ依存のため、ここではスタブとする
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kafka_config_deserialization() {
        let yaml = r#"
brokers:
  - "kafka-0.messaging.svc.cluster.local:9092"
  - "kafka-1.messaging.svc.cluster.local:9092"
consumer_group: "auth-server.default"
security_protocol: "PLAINTEXT"
sasl:
  mechanism: "SCRAM-SHA-512"
  username: "user"
  password: "pass"
topics:
  publish:
    - "k1s0.system.auth.login.v1"
    - "k1s0.system.auth.token_validate.v1"
  subscribe: []
"#;
        let config: KafkaConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.brokers.len(), 2);
        assert_eq!(config.consumer_group, "auth-server.default");
        assert_eq!(config.sasl.mechanism, "SCRAM-SHA-512");
        assert_eq!(config.topics.publish.len(), 2);
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
    async fn test_kafka_producer_send_stub() {
        let config = KafkaConfig {
            brokers: vec!["localhost:9092".to_string()],
            consumer_group: "test".to_string(),
            security_protocol: "PLAINTEXT".to_string(),
            sasl: SaslConfig::default(),
            topics: TopicsConfig::default(),
        };

        let producer = KafkaProducer::new(config);
        let result = producer
            .send_audit_event("test-topic", "key-1", b"test payload")
            .await;
        assert!(result.is_ok());
    }
}
