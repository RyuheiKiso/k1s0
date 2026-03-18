use async_trait::async_trait;

use crate::domain::entity::policy::Policy;

/// PolicyChangedEvent はポリシー変更時に Kafka へ発行するイベント。
#[derive(Debug, serde::Serialize)]
pub struct PolicyChangedEvent {
    pub event_type: String,
    pub policy_id: String,
    pub policy_name: String,
    pub action: String, // "CREATED" | "UPDATED" | "DELETED"
    pub actor_user_id: Option<String>,
    pub before: Option<serde_json::Value>,
    pub after: Option<serde_json::Value>,
    pub version: u32,
    pub timestamp: String, // ISO 8601
}

impl PolicyChangedEvent {
    fn snapshot(policy: &Policy) -> serde_json::Value {
        serde_json::json!({
            "policy_id": policy.id.to_string(),
            "policy_name": policy.name.clone(),
            "version": policy.version,
            "enabled": policy.enabled,
        })
    }

    pub fn created(policy: &Policy) -> Self {
        Self {
            event_type: "POLICY_CHANGED".to_string(),
            policy_id: policy.id.to_string(),
            policy_name: policy.name.clone(),
            action: "CREATED".to_string(),
            actor_user_id: None,
            before: None,
            after: Some(Self::snapshot(policy)),
            version: policy.version,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn updated(before: &Policy, after: &Policy) -> Self {
        Self {
            event_type: "POLICY_CHANGED".to_string(),
            policy_id: after.id.to_string(),
            policy_name: after.name.clone(),
            action: "UPDATED".to_string(),
            actor_user_id: None,
            before: Some(Self::snapshot(before)),
            after: Some(Self::snapshot(after)),
            version: after.version,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn deleted(policy: &Policy) -> Self {
        Self {
            event_type: "POLICY_CHANGED".to_string(),
            policy_id: policy.id.to_string(),
            policy_name: policy.name.clone(),
            action: "DELETED".to_string(),
            actor_user_id: None,
            before: Some(Self::snapshot(policy)),
            after: None,
            version: policy.version,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// PolicyEventPublisher はポリシー変更イベント配信のためのトレイト。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait PolicyEventPublisher: Send + Sync {
    async fn publish_policy_changed(&self, event: &PolicyChangedEvent) -> anyhow::Result<()>;
    #[allow(dead_code)]
    async fn close(&self) -> anyhow::Result<()>;
}

/// NoopPolicyEventPublisher はイベントを破棄する No-Op 実装。
pub struct NoopPolicyEventPublisher;

#[async_trait]
impl PolicyEventPublisher for NoopPolicyEventPublisher {
    async fn publish_policy_changed(&self, _event: &PolicyChangedEvent) -> anyhow::Result<()> {
        Ok(())
    }

    async fn close(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

/// KafkaPolicyProducer は rdkafka FutureProducer を使った Kafka プロデューサー。
pub struct KafkaPolicyProducer {
    producer: rdkafka::producer::FutureProducer,
    topic: String,
    metrics: Option<std::sync::Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl KafkaPolicyProducer {
    /// 新しい KafkaPolicyProducer を作成する。
    pub fn new(config: &crate::infrastructure::config::KafkaConfig) -> anyhow::Result<Self> {
        use rdkafka::config::ClientConfig;

        let topic = config.topic.clone();

        let mut client_config = ClientConfig::new();
        client_config.set("bootstrap.servers", config.brokers.join(","));
        client_config.set("security.protocol", &config.security_protocol);
        client_config.set("acks", "all");
        client_config.set("message.timeout.ms", "5000");
        // 冪等プロデューサーを有効化し、メッセージの重複送信を防止する
        client_config.set("enable.idempotence", "true");

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
    #[allow(dead_code)]
    pub fn topic(&self) -> &str {
        &self.topic
    }
}

#[async_trait]
impl PolicyEventPublisher for KafkaPolicyProducer {
    async fn publish_policy_changed(&self, event: &PolicyChangedEvent) -> anyhow::Result<()> {
        use rdkafka::producer::FutureRecord;
        use std::time::Duration;

        let payload = serde_json::to_vec(event)?;
        let key = event.policy_id.clone();

        let record = FutureRecord::to(&self.topic).key(&key).payload(&payload);

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(err, _)| {
                anyhow::anyhow!("failed to publish policy changed event: {}", err)
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

    #[test]
    fn test_policy_changed_event_created() {
        let policy = Policy {
            id: uuid::Uuid::new_v4(),
            name: "allow-read".to_string(),
            description: "desc".to_string(),
            rego_content: "package authz".to_string(),
            package_path: String::new(),
            bundle_id: None,
            version: 1,
            enabled: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let event = PolicyChangedEvent::created(&policy);
        assert_eq!(event.action, "CREATED");
        assert_eq!(event.policy_name, "allow-read");
        assert_eq!(event.version, 1);
    }

    #[test]
    fn test_policy_changed_event_updated() {
        let before = Policy {
            id: uuid::Uuid::new_v4(),
            name: "allow-read".to_string(),
            description: "desc".to_string(),
            rego_content: "package authz".to_string(),
            package_path: String::new(),
            bundle_id: None,
            version: 2,
            enabled: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let mut after = before.clone();
        after.version = 3;
        after.enabled = false;
        let event = PolicyChangedEvent::updated(&before, &after);
        assert_eq!(event.action, "UPDATED");
        assert_eq!(event.version, 3);
        assert!(event.before.is_some());
    }

    #[test]
    fn test_policy_changed_event_deleted() {
        let policy = Policy {
            id: uuid::Uuid::new_v4(),
            name: "allow-read".to_string(),
            description: "desc".to_string(),
            rego_content: "package authz".to_string(),
            package_path: String::new(),
            bundle_id: None,
            version: 4,
            enabled: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let event = PolicyChangedEvent::deleted(&policy);
        assert_eq!(event.action, "DELETED");
        assert_eq!(event.policy_id, policy.id.to_string());
        assert_eq!(event.version, 4);
    }

    #[test]
    fn test_policy_changed_event_serialization() {
        let policy = Policy {
            id: uuid::Uuid::new_v4(),
            name: "allow-read".to_string(),
            description: "desc".to_string(),
            rego_content: "package authz".to_string(),
            package_path: String::new(),
            bundle_id: None,
            version: 1,
            enabled: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let event = PolicyChangedEvent::created(&policy);
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["action"], "CREATED");
        assert_eq!(json["policy_name"], "allow-read");
        assert_eq!(json["version"], 1);
    }

    #[tokio::test]
    async fn test_noop_publisher() {
        let publisher = NoopPolicyEventPublisher;
        let policy = Policy {
            id: uuid::Uuid::new_v4(),
            name: "test".to_string(),
            description: "test".to_string(),
            rego_content: "package test".to_string(),
            package_path: String::new(),
            bundle_id: None,
            version: 1,
            enabled: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let event = PolicyChangedEvent::created(&policy);
        assert!(publisher.publish_policy_changed(&event).await.is_ok());
        assert!(publisher.close().await.is_ok());
    }

    #[tokio::test]
    async fn test_mock_publisher() {
        let mut mock = MockPolicyEventPublisher::new();
        mock.expect_publish_policy_changed().returning(|_| Ok(()));
        mock.expect_close().returning(|| Ok(()));

        let policy = Policy {
            id: uuid::Uuid::new_v4(),
            name: "test".to_string(),
            description: "test".to_string(),
            rego_content: "package test".to_string(),
            package_path: String::new(),
            bundle_id: None,
            version: 1,
            enabled: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let event = PolicyChangedEvent::created(&policy);
        assert!(mock.publish_policy_changed(&event).await.is_ok());
        assert!(mock.close().await.is_ok());
    }
}
