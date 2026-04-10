use async_trait::async_trait;
// secrecy クレートを使用して Kafka SASL パスワードを Secret<String> で保持し、Debug 出力への漏洩を防ぐ（H-1 監査対応）。
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

use crate::domain::entity::audit_log::AuditLog;

/// `KafkaConfig` は Kafka 接続の設定を表す。
#[allow(dead_code)]
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

/// セキュリティデフォルト: 本番環境では `SASL_SSL` を強制する。
/// 開発環境では config.dev.yaml / config.docker.yaml で明示的に PLAINTEXT を指定すること。
fn default_security_protocol() -> String {
    "SASL_SSL".to_string()
}

/// `SaslConfig` は SASL 認証の設定を表す。
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

/// `TopicsConfig` はトピック設定を表す。
#[allow(dead_code)]
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TopicsConfig {
    #[serde(default)]
    pub publish: Vec<String>,
    #[serde(default)]
    pub subscribe: Vec<String>,
}

/// `AuditEventPublisher` は監査イベント配信のためのトレイト。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AuditEventPublisher: Send + Sync {
    async fn publish(&self, event: &AuditLog) -> anyhow::Result<()>;
    #[allow(dead_code)]
    async fn close(&self) -> anyhow::Result<()>;
}

/// `KafkaProducer` is a Kafka producer using rdkafka `FutureProducer`.
pub struct KafkaProducer {
    producer: rdkafka::producer::FutureProducer,
    topic: String,
    topic_map: std::collections::HashMap<String, String>,
}

impl KafkaProducer {
    /// Create a new `KafkaProducer`.
    pub fn new(config: &KafkaConfig) -> anyhow::Result<Self> {
        use rdkafka::config::ClientConfig;

        let topic = config
            .topics
            .publish
            .first()
            .cloned()
            .unwrap_or_else(|| "k1s0.system.auth.audit.v1".to_string());

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

        let topic_map = Self::build_default_topic_map();

        Ok(Self {
            producer,
            topic,
            topic_map,
        })
    }

    /// Return the default topic name.
    #[allow(dead_code)]
    #[must_use]
    pub fn topic(&self) -> &str {
        &self.topic
    }

    /// Resolve a topic by event type prefix, falling back to the default topic.
    #[must_use]
    pub fn resolve_topic(&self, event_type: &str) -> &str {
        let prefix = event_type.split('_').next().unwrap_or("");
        self.topic_map
            .get(prefix)
            .map_or(&self.topic, std::string::String::as_str)
    }

    fn build_default_topic_map() -> std::collections::HashMap<String, String> {
        let mut map = std::collections::HashMap::new();
        map.insert("LOGIN".to_string(), "k1s0.system.auth.login.v1".to_string());
        map.insert(
            "TOKEN".to_string(),
            "k1s0.system.auth.token_validate.v1".to_string(),
        );
        map.insert(
            "PERMISSION".to_string(),
            "k1s0.system.auth.permission_denied.v1".to_string(),
        );
        map
    }
}

#[async_trait]
impl AuditEventPublisher for KafkaProducer {
    async fn publish(&self, event: &AuditLog) -> anyhow::Result<()> {
        use rdkafka::producer::FutureRecord;
        use std::time::Duration;

        let payload = serde_json::to_vec(event)?;
        let key = &event.user_id;
        let target_topic = self.resolve_topic(&event.event_type);

        let record = FutureRecord::to(target_topic).key(key).payload(&payload);

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(err, _)| anyhow::anyhow!("failed to publish audit event: {err}"))?;

        Ok(())
    }

    async fn close(&self) -> anyhow::Result<()> {
        use rdkafka::producer::Producer;
        self.producer.flush(std::time::Duration::from_secs(5))?;
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::audit_log::{AuditLog, CreateAuditLogRequest};
    use std::sync::Mutex;

    // MED-023 監査対応: std::sync::Mutex を async テスト内で使用しているが、
    // このテストでは Mutex のロックを await ポイントをまたいで保持していないため問題ない。
    // lock() → push() → drop という同期操作のみであり、tokio::Mutex への変更は不要。
    // 参考: https://tokio.rs/tokio/tutorial/shared-state#on-using-stdsyncmutex
    //
    // LOW-013 設計注記（InMemoryProducer 共通化の検討）:
    // 11サービスで類似した InMemoryProducer が重複実装されているが、各サービスは
    // 独自の AuditEventPublisher トレイトを定義しているため server-common への移動には
    // トレイト定義の共通化も必要となり影響範囲が大きい。
    // 将来課題: k1s0-server-common に testing フィーチャーを追加し、
    // AuditEventPublisher トレイトと InMemoryProducer を共通モジュールに集約することで
    // `use k1s0_server_common::testing::InMemoryProducer;` で参照できるようにする。

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
    impl AuditEventPublisher for InMemoryProducer {
        async fn publish(&self, event: &AuditLog) -> anyhow::Result<()> {
            if self.should_fail {
                return Err(anyhow::anyhow!("broker connection refused"));
            }
            let payload = serde_json::to_vec(event)?;
            let key = event.user_id.clone();
            self.messages.lock().unwrap().push((key, payload));
            Ok(())
        }

        async fn close(&self) -> anyhow::Result<()> {
            Ok(())
        }
    }

    fn make_test_audit_log() -> AuditLog {
        AuditLog::new(CreateAuditLogRequest {
            event_type: "LOGIN_SUCCESS".to_string(),
            user_id: "user-uuid-5678".to_string(),
            ip_address: "192.168.1.100".to_string(),
            user_agent: "Mozilla/5.0".to_string(),
            resource: "/api/v1/auth/token".to_string(),
            action: "POST".to_string(),
            result: "SUCCESS".to_string(),
            resource_id: None,
            detail: Some(serde_json::json!({"client_id": "react-spa"})),
            trace_id: None,
        })
    }

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
        assert_eq!(config.security_protocol, "SASL_SSL");
        assert!(config.consumer_group.is_empty());
        assert!(config.topics.publish.is_empty());
    }

    #[tokio::test]
    async fn test_publish_serialization() {
        let producer = InMemoryProducer::new();
        let log = make_test_audit_log();

        let result = producer.publish(&log).await;
        assert!(result.is_ok());

        let messages = producer.messages.lock().unwrap();
        assert_eq!(messages.len(), 1);

        // JSON に正常変換されていることを確認
        let deserialized: AuditLog = serde_json::from_slice(&messages[0].1).unwrap();
        assert_eq!(deserialized.event_type, "LOGIN_SUCCESS");
        assert_eq!(deserialized.user_id, "user-uuid-5678");
        assert_eq!(deserialized.result, "SUCCESS");
        let detail = deserialized.detail.as_ref().unwrap();
        assert_eq!(detail["client_id"], "react-spa");
    }

    #[tokio::test]
    async fn test_publish_key_is_user_id() {
        let producer = InMemoryProducer::new();
        let log = make_test_audit_log();

        producer.publish(&log).await.unwrap();

        let messages = producer.messages.lock().unwrap();
        assert_eq!(messages.len(), 1);
        // パーティションキーが user_id であることを確認
        assert_eq!(messages[0].0, "user-uuid-5678");
    }

    #[tokio::test]
    async fn test_publish_connection_error() {
        let producer = InMemoryProducer::with_error();
        let log = make_test_audit_log();

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
        // publish トピックが未設定時のデフォルトトピック名を検証
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
            .unwrap_or_else(|| "k1s0.system.auth.audit.v1".to_string());
        assert_eq!(topic, "k1s0.system.auth.audit.v1");
    }

    #[test]
    fn test_configured_topic_name() {
        let yaml = r#"
brokers:
  - "localhost:9092"
topics:
  publish:
    - "k1s0.system.auth.audit.v1"
  subscribe: []
"#;
        let config: KafkaConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.topics.publish[0], "k1s0.system.auth.audit.v1");
    }

    #[tokio::test]
    async fn test_mock_audit_event_publisher() {
        let mut mock = MockAuditEventPublisher::new();
        mock.expect_publish().returning(|_| Ok(()));
        mock.expect_close().returning(|| Ok(()));

        let log = make_test_audit_log();
        assert!(mock.publish(&log).await.is_ok());
        assert!(mock.close().await.is_ok());
    }
}
