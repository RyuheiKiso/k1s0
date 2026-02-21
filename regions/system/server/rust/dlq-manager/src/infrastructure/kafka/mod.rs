pub mod consumer;
pub mod producer;

use serde::Deserialize;

/// KafkaConfig は Kafka 接続の設定を表す。
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

fn default_security_protocol() -> String {
    "PLAINTEXT".to_string()
}

fn default_dlq_topic_pattern() -> String {
    "*.dlq.v1".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kafka_config_deserialization() {
        let yaml = r#"
brokers:
  - "kafka-0.messaging.svc.cluster.local:9092"
consumer_group: "dlq-manager.default"
security_protocol: "PLAINTEXT"
dlq_topic_pattern: "*.dlq.v1"
"#;
        let config: KafkaConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.brokers.len(), 1);
        assert_eq!(config.consumer_group, "dlq-manager.default");
        assert_eq!(config.dlq_topic_pattern, "*.dlq.v1");
    }

    #[test]
    fn test_kafka_config_defaults() {
        let yaml = r#"
brokers:
  - "localhost:9092"
"#;
        let config: KafkaConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.security_protocol, "PLAINTEXT");
        assert_eq!(config.dlq_topic_pattern, "*.dlq.v1");
    }
}
