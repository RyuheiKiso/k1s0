use async_trait::async_trait;
use serde::Deserialize;

use crate::domain::entity::Tenant;

/// TenantChangedEvent はテナント変更時に Kafka へ発行するイベント。
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TenantChangedEvent {
    pub tenant_id: String,
    pub tenant_name: String,
    pub action: String,
    pub status: String,
    pub timestamp: String, // ISO 8601
}

impl TenantChangedEvent {
    pub fn from_tenant(tenant: &Tenant, action: &str) -> Self {
        Self {
            tenant_id: tenant.id.to_string(),
            tenant_name: tenant.name.clone(),
            action: action.to_string(),
            status: tenant.status.as_str().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
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

const DEFAULT_TOPIC: &str = "k1s0.system.tenant.changed.v1";

/// TenantEventPublisher はテナント変更イベント配信のためのトレイト。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait TenantEventPublisher: Send + Sync {
    async fn publish_tenant_created(&self, tenant: &Tenant) -> anyhow::Result<()>;
    async fn publish_tenant_updated(&self, tenant: &Tenant) -> anyhow::Result<()>;
    async fn close(&self) -> anyhow::Result<()>;
}

/// NoopTenantEventPublisher はイベント発行を行わない実装。
/// Kafka 未設定時のフォールバックとして利用する。
pub struct NoopTenantEventPublisher;

#[async_trait]
impl TenantEventPublisher for NoopTenantEventPublisher {
    async fn publish_tenant_created(&self, _tenant: &Tenant) -> anyhow::Result<()> {
        tracing::debug!("noop: skipping tenant_created event");
        Ok(())
    }

    async fn publish_tenant_updated(&self, _tenant: &Tenant) -> anyhow::Result<()> {
        tracing::debug!("noop: skipping tenant_updated event");
        Ok(())
    }

    async fn close(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

/// KafkaTenantEventPublisher は rdkafka FutureProducer を使った Kafka プロデューサー。
pub struct KafkaTenantEventPublisher {
    producer: rdkafka::producer::FutureProducer,
    topic: String,
    metrics: Option<std::sync::Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl KafkaTenantEventPublisher {
    /// 新しい KafkaTenantEventPublisher を作成する。
    pub fn new(config: &KafkaConfig) -> anyhow::Result<Self> {
        use rdkafka::config::ClientConfig;

        let topic = config
            .topics
            .publish
            .first()
            .cloned()
            .unwrap_or_else(|| DEFAULT_TOPIC.to_string());

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

    /// イベントを Kafka へ発行する内部メソッド。
    async fn publish_event(&self, event: &TenantChangedEvent) -> anyhow::Result<()> {
        use rdkafka::producer::FutureRecord;
        use std::time::Duration;

        let payload = serde_json::to_vec(event)?;
        let key = format!("tenant:{}", event.tenant_id);

        let record = FutureRecord::to(&self.topic).key(&key).payload(&payload);

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(err, _)| {
                anyhow::anyhow!("failed to publish tenant event: {}", err)
            })?;

        if let Some(ref m) = self.metrics {
            m.record_kafka_message_produced(&self.topic);
        }

        Ok(())
    }
}

#[async_trait]
impl TenantEventPublisher for KafkaTenantEventPublisher {
    async fn publish_tenant_created(&self, tenant: &Tenant) -> anyhow::Result<()> {
        let event = TenantChangedEvent::from_tenant(tenant, "created");
        self.publish_event(&event).await
    }

    async fn publish_tenant_updated(&self, tenant: &Tenant) -> anyhow::Result<()> {
        let event = TenantChangedEvent::from_tenant(tenant, "updated");
        self.publish_event(&event).await
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
    use crate::domain::entity::TenantStatus;
    use std::sync::Mutex;

    /// テスト用のインメモリプロデューサー。
    struct InMemoryPublisher {
        messages: Mutex<Vec<(String, Vec<u8>)>>,
        should_fail: bool,
    }

    impl InMemoryPublisher {
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
    impl TenantEventPublisher for InMemoryPublisher {
        async fn publish_tenant_created(&self, tenant: &Tenant) -> anyhow::Result<()> {
            if self.should_fail {
                return Err(anyhow::anyhow!("broker connection refused"));
            }
            let event = TenantChangedEvent::from_tenant(tenant, "created");
            let payload = serde_json::to_vec(&event)?;
            let key = format!("tenant:{}", tenant.id);
            self.messages.lock().unwrap().push((key, payload));
            Ok(())
        }

        async fn publish_tenant_updated(&self, tenant: &Tenant) -> anyhow::Result<()> {
            if self.should_fail {
                return Err(anyhow::anyhow!("broker connection refused"));
            }
            let event = TenantChangedEvent::from_tenant(tenant, "updated");
            let payload = serde_json::to_vec(&event)?;
            let key = format!("tenant:{}", tenant.id);
            self.messages.lock().unwrap().push((key, payload));
            Ok(())
        }

        async fn close(&self) -> anyhow::Result<()> {
            Ok(())
        }
    }

    fn make_test_tenant() -> Tenant {
        Tenant {
            id: uuid::Uuid::new_v4(),
            name: "acme-corp".to_string(),
            display_name: "ACME Corporation".to_string(),
            status: TenantStatus::Active,
            plan: "professional".to_string(),
            created_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_kafka_config_deserialization() {
        let yaml = r#"
brokers:
  - "kafka-0.messaging.svc.cluster.local:9092"
  - "kafka-1.messaging.svc.cluster.local:9092"
consumer_group: "tenant-server.default"
security_protocol: "PLAINTEXT"
sasl:
  mechanism: "SCRAM-SHA-512"
  username: "user"
  password: "pass"
topics:
  publish:
    - "k1s0.system.tenant.changed.v1"
  subscribe: []
"#;
        let config: KafkaConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.brokers.len(), 2);
        assert_eq!(config.consumer_group, "tenant-server.default");
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
            .unwrap_or_else(|| DEFAULT_TOPIC.to_string());
        assert_eq!(topic, "k1s0.system.tenant.changed.v1");
    }

    #[test]
    fn test_tenant_changed_event_serialization() {
        let tenant = make_test_tenant();
        let event = TenantChangedEvent::from_tenant(&tenant, "created");
        let json = serde_json::to_value(&event).unwrap();

        assert_eq!(json["tenant_id"], tenant.id.to_string());
        assert_eq!(json["tenant_name"], "acme-corp");
        assert_eq!(json["action"], "created");
        assert_eq!(json["status"], "active");
        assert!(json["timestamp"].as_str().is_some());
    }

    #[test]
    fn test_tenant_changed_event_debug_format() {
        let tenant = make_test_tenant();
        let event = TenantChangedEvent::from_tenant(&tenant, "updated");
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("TenantChangedEvent"));
        assert!(debug_str.contains("acme-corp"));
    }

    #[tokio::test]
    async fn test_publish_tenant_created() {
        let publisher = InMemoryPublisher::new();
        let tenant = make_test_tenant();

        let result = publisher.publish_tenant_created(&tenant).await;
        assert!(result.is_ok());

        let messages = publisher.messages.lock().unwrap();
        assert_eq!(messages.len(), 1);

        let deserialized: TenantChangedEvent =
            serde_json::from_slice(&messages[0].1).unwrap();
        assert_eq!(deserialized.action, "created");
        assert_eq!(deserialized.tenant_name, "acme-corp");
    }

    #[tokio::test]
    async fn test_publish_tenant_updated() {
        let publisher = InMemoryPublisher::new();
        let tenant = make_test_tenant();

        let result = publisher.publish_tenant_updated(&tenant).await;
        assert!(result.is_ok());

        let messages = publisher.messages.lock().unwrap();
        assert_eq!(messages.len(), 1);

        let deserialized: TenantChangedEvent =
            serde_json::from_slice(&messages[0].1).unwrap();
        assert_eq!(deserialized.action, "updated");
    }

    #[tokio::test]
    async fn test_publish_key_format() {
        let publisher = InMemoryPublisher::new();
        let tenant = make_test_tenant();

        publisher.publish_tenant_created(&tenant).await.unwrap();

        let messages = publisher.messages.lock().unwrap();
        assert_eq!(messages[0].0, format!("tenant:{}", tenant.id));
    }

    #[tokio::test]
    async fn test_publish_connection_error() {
        let publisher = InMemoryPublisher::with_error();
        let tenant = make_test_tenant();

        let result = publisher.publish_tenant_created(&tenant).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("broker connection refused"));
    }

    #[tokio::test]
    async fn test_noop_publisher() {
        let noop = NoopTenantEventPublisher;
        let tenant = make_test_tenant();

        assert!(noop.publish_tenant_created(&tenant).await.is_ok());
        assert!(noop.publish_tenant_updated(&tenant).await.is_ok());
        assert!(noop.close().await.is_ok());
    }

    #[tokio::test]
    async fn test_close_graceful() {
        let publisher = InMemoryPublisher::new();
        let result = publisher.close().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_tenant_event_publisher() {
        let mut mock = MockTenantEventPublisher::new();
        mock.expect_publish_tenant_created().returning(|_| Ok(()));
        mock.expect_publish_tenant_updated().returning(|_| Ok(()));
        mock.expect_close().returning(|| Ok(()));

        let tenant = make_test_tenant();
        assert!(mock.publish_tenant_created(&tenant).await.is_ok());
        assert!(mock.publish_tenant_updated(&tenant).await.is_ok());
        assert!(mock.close().await.is_ok());
    }
}
