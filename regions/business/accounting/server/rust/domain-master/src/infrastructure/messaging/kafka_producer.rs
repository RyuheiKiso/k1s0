use crate::infrastructure::config::KafkaConfig;
use crate::usecase::event_publisher::DomainMasterEventPublisher;
use async_trait::async_trait;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde_json::Value;
use std::time::Duration;

pub struct DomainMasterKafkaProducer {
    producer: FutureProducer,
    category_topic: String,
    item_topic: String,
    tenant_extension_topic: String,
}

impl DomainMasterKafkaProducer {
    pub fn new(config: &KafkaConfig) -> anyhow::Result<Self> {
        let mut client_config = ClientConfig::new();
        client_config.set("bootstrap.servers", config.brokers.join(","));
        client_config.set("acks", "all");
        client_config.set("message.timeout.ms", "5000");
        // 冪等プロデューサーを有効化し、メッセージの重複送信を防止する
        client_config.set("enable.idempotence", "true");

        let producer = client_config.create()?;

        Ok(Self {
            producer,
            category_topic: config.category_topic.clone(),
            item_topic: config.item_topic.clone(),
            tenant_extension_topic: config.tenant_extension_topic.clone(),
        })
    }

    pub fn category_topic(&self) -> &str {
        &self.category_topic
    }

    pub fn item_topic(&self) -> &str {
        &self.item_topic
    }

    pub fn tenant_extension_topic(&self) -> &str {
        &self.tenant_extension_topic
    }

    pub async fn publish_category_changed(&self, event: &Value) -> anyhow::Result<()> {
        self.publish(&self.category_topic, event).await
    }

    pub async fn publish_item_changed(&self, event: &Value) -> anyhow::Result<()> {
        self.publish(&self.item_topic, event).await
    }

    pub async fn publish_tenant_extension_changed(&self, event: &Value) -> anyhow::Result<()> {
        self.publish(&self.tenant_extension_topic, event).await
    }

    async fn publish(&self, topic: &str, event: &Value) -> anyhow::Result<()> {
        let payload = serde_json::to_vec(event)?;
        // イベントタイプに応じたキー選択: item → item_code, tenant → tenant_id, category → category_code
        let key = event
            .get("item_code")
            .and_then(Value::as_str)
            .or_else(|| event.get("tenant_id").and_then(Value::as_str))
            .or_else(|| event.get("category_code").and_then(Value::as_str))
            .unwrap_or("domain-master");

        tracing::info!(topic = %topic, key, "publishing change event");

        self.producer
            .send(
                FutureRecord::to(topic).key(key).payload(&payload),
                Duration::from_secs(5),
            )
            .await
            .map_err(|(err, _)| anyhow::anyhow!("failed to publish change event: {err}"))?;

        Ok(())
    }
}

#[async_trait]
impl DomainMasterEventPublisher for DomainMasterKafkaProducer {
    async fn publish_category_changed(&self, event: &Value) -> anyhow::Result<()> {
        DomainMasterKafkaProducer::publish_category_changed(self, event).await
    }

    async fn publish_item_changed(&self, event: &Value) -> anyhow::Result<()> {
        DomainMasterKafkaProducer::publish_item_changed(self, event).await
    }

    async fn publish_tenant_extension_changed(&self, event: &Value) -> anyhow::Result<()> {
        DomainMasterKafkaProducer::publish_tenant_extension_changed(self, event).await
    }
}
