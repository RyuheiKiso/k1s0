// secrecy クレートを使用して Kafka SASL パスワードを Secret<String> で保持し、Debug 出力への漏洩を防ぐ（H-1 監査対応）。
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

/// ConfigChangedEvent は設定値変更時に Kafka へ発行するイベント。
#[derive(Debug, serde::Serialize)]
pub struct ConfigChangedEvent {
    pub event_type: String,
    pub namespace: String,
    pub key: String,
    pub actor_user_id: String,
    pub before: Option<serde_json::Value>,
    pub after: serde_json::Value,
    pub new_value: serde_json::Value,
    pub updated_by: String,
    pub version: i32,
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
    #[serde(default, alias = "topic_changed", alias = "topic")]
    pub topic_changed: String,
}

/// セキュリティデフォルト: 本番環境では SASL_SSL を強制する。
/// 開発環境では config.dev.yaml / config.docker.yaml で明示的に PLAINTEXT を指定すること。
fn default_security_protocol() -> String {
    "SASL_SSL".to_string()
}

/// SaslConfig は SASL 認証の設定を表す。
#[derive(Debug, Clone, Deserialize)]
pub struct SaslConfig {
    #[serde(default)]
    pub mechanism: String,
    #[serde(default)]
    pub username: String,
    // SASL パスワードは Secret<String> で保持し、Debug トレイトでは [REDACTED] と表示される
    pub password: Secret<String>,
}

impl Default for SaslConfig {
    fn default() -> Self {
        Self {
            mechanism: String::new(),
            username: String::new(),
            // デフォルト値は空文字列で初期化する
            password: Secret::new(String::new()),
        }
    }
}

/// TopicsConfig はトピック設定を表す。
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TopicsConfig {
    #[serde(default)]
    pub publish: Vec<String>,
    #[serde(default)]
    pub subscribe: Vec<String>,
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

        let topic = if !config.topic_changed.is_empty() {
            config.topic_changed.clone()
        } else {
            config
                .topics
                .publish
                .first()
                .cloned()
                .unwrap_or_else(|| "k1s0.system.config.changed.v1".to_string())
        };

        let mut client_config = ClientConfig::new();
        client_config.set("bootstrap.servers", config.brokers.join(","));
        client_config.set("security.protocol", &config.security_protocol);
        client_config.set("acks", "all");
        client_config.set("message.timeout.ms", "5000");
        // 冪等プロデューサーを有効化し、メッセージの重複送信を防止する
        client_config.set("enable.idempotence", "true");

        if !config.sasl.mechanism.is_empty() {
            client_config.set("sasl.mechanism", &config.sasl.mechanism);
            client_config.set("sasl.username", &config.sasl.username);
            // expose_secret() で SASL パスワードを取り出して rdkafka に設定する
            client_config.set("sasl.password", config.sasl.password.expose_secret());
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
    /// 設定値変更イベントを Kafka へ発行する。
    /// 内部的には ConfigChangeLog を構築して既存の publish メソッドに委譲する。
    pub async fn publish_config_changed(&self, event: &ConfigChangedEvent) -> anyhow::Result<()> {
        use rdkafka::producer::FutureRecord;
        use std::time::Duration;

        let payload = serde_json::to_vec(event)?;
        let key = format!("{}:{}", event.namespace, event.key);

        let record = FutureRecord::to(&self.topic).key(&key).payload(&payload);

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(err, _)| {
                anyhow::anyhow!("failed to publish config changed event: {}", err)
            })?;

        if let Some(ref m) = self.metrics {
            m.record_kafka_message_produced(&self.topic);
        }

        Ok(())
    }
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
        assert_eq!(config.security_protocol, "SASL_SSL");
        assert!(config.consumer_group.is_empty());
        assert!(config.topics.publish.is_empty());
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

    // --- ConfigChangedEvent テスト ---

    fn make_config_changed_event() -> ConfigChangedEvent {
        ConfigChangedEvent {
            event_type: "CONFIG_CHANGED".to_string(),
            namespace: "system.auth.database".to_string(),
            key: "max_connections".to_string(),
            actor_user_id: "operator@example.com".to_string(),
            before: Some(serde_json::json!(40)),
            after: serde_json::json!(50),
            new_value: serde_json::json!(50),
            updated_by: "operator@example.com".to_string(),
            version: 4,
            timestamp: "2026-02-21T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_config_changed_event_serialization() {
        let event = make_config_changed_event();
        let json = serde_json::to_value(&event).unwrap();

        assert_eq!(json["namespace"], "system.auth.database");
        assert_eq!(json["key"], "max_connections");
        assert_eq!(json["event_type"], "CONFIG_CHANGED");
        assert_eq!(json["new_value"], 50);
        assert_eq!(json["updated_by"], "operator@example.com");
        assert_eq!(json["version"], 4);
        assert_eq!(json["timestamp"], "2026-02-21T00:00:00Z");
    }

    #[test]
    fn test_config_changed_event_serialization_object_value() {
        let event = ConfigChangedEvent {
            event_type: "CONFIG_CHANGED".to_string(),
            namespace: "system.auth.jwt".to_string(),
            key: "settings".to_string(),
            actor_user_id: "admin@example.com".to_string(),
            before: None,
            after: serde_json::json!({ "ttl_secs": 7200, "issuer": "https://auth.example.com" }),
            new_value: serde_json::json!({ "ttl_secs": 7200, "issuer": "https://auth.example.com" }),
            updated_by: "admin@example.com".to_string(),
            version: 2,
            timestamp: "2026-02-21T12:00:00Z".to_string(),
        };

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["new_value"]["ttl_secs"], 7200);
        assert_eq!(json["new_value"]["issuer"], "https://auth.example.com");
    }

    #[test]
    fn test_config_changed_event_debug_format() {
        let event = make_config_changed_event();
        // Debug トレイトが実装されていることを確認
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("ConfigChangedEvent"));
        assert!(debug_str.contains("system.auth.database"));
    }
}
