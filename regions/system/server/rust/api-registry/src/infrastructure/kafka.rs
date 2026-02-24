//! Kafka プロデューサー実装。
//! SchemaUpdatedEvent を k1s0.system.apiregistry.schema_updated.v1 トピックに送信する。

use async_trait::async_trait;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde::Serialize;
use std::time::Duration;

#[derive(Debug, Serialize)]
pub struct SchemaUpdatedEvent {
    pub event_type: String,
    pub schema_name: String,
    pub schema_type: String,
    pub version: u32,
    pub content_hash: Option<String>,
    pub breaking_changes: Option<bool>,
    pub registered_by: Option<String>,
    pub deleted_by: Option<String>,
    pub timestamp: String,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SchemaEventPublisher: Send + Sync {
    async fn publish_schema_updated(&self, event: &SchemaUpdatedEvent) -> anyhow::Result<()>;
}

pub struct KafkaSchemaEventPublisher {
    producer: FutureProducer,
    topic: String,
}

impl KafkaSchemaEventPublisher {
    pub fn new(brokers: &[String], topic: &str) -> anyhow::Result<Self> {
        let producer = ClientConfig::new()
            .set("bootstrap.servers", brokers.join(","))
            .set("message.timeout.ms", "5000")
            .set("acks", "all")
            .create()?;
        Ok(Self {
            producer,
            topic: topic.to_string(),
        })
    }
}

#[async_trait]
impl SchemaEventPublisher for KafkaSchemaEventPublisher {
    async fn publish_schema_updated(&self, event: &SchemaUpdatedEvent) -> anyhow::Result<()> {
        let payload = serde_json::to_string(event)?;
        let record = FutureRecord::to(&self.topic)
            .payload(&payload)
            .key(&event.schema_name);
        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(err, _)| anyhow::anyhow!("Kafka send error: {}", err))?;
        Ok(())
    }
}

pub struct NoopSchemaEventPublisher;

#[async_trait]
impl SchemaEventPublisher for NoopSchemaEventPublisher {
    async fn publish_schema_updated(&self, _event: &SchemaUpdatedEvent) -> anyhow::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_noop_publisher_succeeds() {
        let publisher = NoopSchemaEventPublisher;
        let event = SchemaUpdatedEvent {
            event_type: "SCHEMA_VERSION_REGISTERED".to_string(),
            schema_name: "test-api".to_string(),
            schema_type: "openapi".to_string(),
            version: 1,
            content_hash: Some("sha256:abc123".to_string()),
            breaking_changes: Some(false),
            registered_by: Some("user-001".to_string()),
            deleted_by: None,
            timestamp: "2026-02-24T00:00:00Z".to_string(),
        };
        let result = publisher.publish_schema_updated(&event).await;
        assert!(result.is_ok());
    }
}
