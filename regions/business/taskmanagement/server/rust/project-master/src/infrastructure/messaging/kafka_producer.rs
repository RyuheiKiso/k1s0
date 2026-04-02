// プロジェクトマスタ Kafka プロデューサー実装。
// rdkafka を使用してプロジェクトマスタ変更イベントを発行する。
use async_trait::async_trait;
use rdkafka::{
    config::ClientConfig,
    producer::{FutureProducer, FutureRecord},
};
use std::time::Duration;

use crate::infrastructure::config::KafkaConfig;
use crate::usecase::event_publisher::{
    ProjectMasterEventPublisher, ProjectTypeChangedEvent, StatusDefinitionChangedEvent,
    TenantExtensionChangedEvent,
};

pub struct ProjectMasterKafkaProducer {
    producer: FutureProducer,
    project_type_changed_topic: String,
    status_definition_changed_topic: String,
    tenant_extension_changed_topic: String,
}

impl ProjectMasterKafkaProducer {
    /// Kafka プロデューサーを初期化する
    pub fn new(config: &KafkaConfig) -> anyhow::Result<Self> {
        let mut client_config = ClientConfig::new();
        client_config.set("bootstrap.servers", config.brokers.join(","));
        client_config.set("security.protocol", &config.security_protocol);
        client_config.set("acks", "all");
        client_config.set("message.timeout.ms", "5000");
        // 冪等プロデューサーを有効化
        client_config.set("enable.idempotence", "true");
        let producer: FutureProducer = client_config.create()?;
        Ok(Self {
            producer,
            project_type_changed_topic: config.project_type_changed_topic.clone(),
            status_definition_changed_topic: config.status_definition_changed_topic.clone(),
            tenant_extension_changed_topic: config.tenant_extension_changed_topic.clone(),
        })
    }

    /// 指定トピックにペイロードを発行する
    async fn publish(&self, topic: &str, key: &str, payload: &[u8]) -> anyhow::Result<()> {
        tracing::info!(topic = %topic, key, "publishing project-master event");
        self.producer
            .send(
                FutureRecord::to(topic).key(key).payload(payload),
                Duration::from_secs(5),
            )
            .await
            .map_err(|(err, _)| anyhow::anyhow!("failed to publish: {err}"))?;
        Ok(())
    }
}

#[async_trait]
impl ProjectMasterEventPublisher for ProjectMasterKafkaProducer {
    async fn publish_project_type_changed(&self, event: &ProjectTypeChangedEvent) -> anyhow::Result<()> {
        // HIGH-013: シリアライズ失敗時は空ペイロードを送信せずエラーを伝播させる
        let payload = serde_json::to_vec(event)
            .map_err(|e| anyhow::anyhow!("Kafkaイベントのシリアライズに失敗: {}", e))?;
        self.publish(&self.project_type_changed_topic, &event.project_type_id, &payload).await
    }

    async fn publish_status_definition_changed(&self, event: &StatusDefinitionChangedEvent) -> anyhow::Result<()> {
        // HIGH-013: シリアライズ失敗時は空ペイロードを送信せずエラーを伝播させる
        let payload = serde_json::to_vec(event)
            .map_err(|e| anyhow::anyhow!("Kafkaイベントのシリアライズに失敗: {}", e))?;
        self.publish(&self.status_definition_changed_topic, &event.status_definition_id, &payload).await
    }

    async fn publish_tenant_extension_changed(&self, event: &TenantExtensionChangedEvent) -> anyhow::Result<()> {
        // HIGH-013: シリアライズ失敗時は空ペイロードを送信せずエラーを伝播させる
        let payload = serde_json::to_vec(event)
            .map_err(|e| anyhow::anyhow!("Kafkaイベントのシリアライズに失敗: {}", e))?;
        self.publish(&self.tenant_extension_changed_topic, &event.tenant_id, &payload).await
    }
}
