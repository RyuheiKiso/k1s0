pub mod consumer;
pub mod producer;

use serde::Deserialize;

/// `KafkaConfig` は Kafka 接続の設定を表す。
#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    pub brokers: Vec<String>,
    #[serde(default)]
    pub consumer_group: String,
    #[serde(default = "default_security_protocol")]
    pub security_protocol: String,
    #[serde(default = "default_dlq_topic_pattern")]
    pub dlq_topic_pattern: String,
}

/// セキュリティデフォルト: 本番環境では `SASL_SSL` を強制する。
/// 開発環境では config.dev.yaml / config.docker.yaml で明示的に PLAINTEXT を指定すること。
fn default_security_protocol() -> String {
    "SASL_SSL".to_string()
}

/// CRIT-005 監査対応: librdkafka の `subscribe()` は ^ プレフィックス付き文字列を正規表現として扱う。
/// glob パターン（*.v1.dlq）はリテラルトピック名として解釈されるため、実際の DLQ トピックに一致しない。
fn default_dlq_topic_pattern() -> String {
    "^.*\\.v1\\.dlq$".to_string()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_kafka_config_deserialization() {
        let yaml = r#"
brokers:
  - "kafka-0.messaging.svc.cluster.local:9092"
consumer_group: "dlq-manager.default"
security_protocol: "PLAINTEXT"
dlq_topic_pattern: "^.*\\.v1\\.dlq$"
"#;
        let config: KafkaConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.brokers.len(), 1);
        assert_eq!(config.consumer_group, "dlq-manager.default");
        assert_eq!(config.dlq_topic_pattern, "^.*\\.v1\\.dlq$");
    }

    #[test]
    fn test_kafka_config_defaults() {
        let yaml = r#"
brokers:
  - "localhost:9092"
"#;
        let config: KafkaConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.security_protocol, "SASL_SSL");
        assert_eq!(config.dlq_topic_pattern, "^.*\\.v1\\.dlq$");
    }
}
