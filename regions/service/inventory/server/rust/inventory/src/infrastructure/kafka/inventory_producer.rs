use crate::infrastructure::config::KafkaConfig;
use crate::usecase::event_publisher::InventoryEventPublisher;
use async_trait::async_trait;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde_json::Value;
use std::time::Duration;

pub struct InventoryKafkaProducer {
    producer: FutureProducer,
    inventory_reserved_topic: String,
    inventory_released_topic: String,
}

impl InventoryKafkaProducer {
    pub fn new(config: &KafkaConfig) -> anyhow::Result<Self> {
        let mut client_config = ClientConfig::new();
        client_config.set("bootstrap.servers", config.brokers.join(","));
        client_config.set("security.protocol", &config.security_protocol);
        client_config.set("acks", "all");
        client_config.set("message.timeout.ms", "5000");

        let producer = client_config.create()?;

        Ok(Self {
            producer,
            inventory_reserved_topic: config.inventory_reserved_topic.clone(),
            inventory_released_topic: config.inventory_released_topic.clone(),
        })
    }

    async fn publish(&self, topic: &str, event: &Value) -> anyhow::Result<()> {
        let payload = serde_json::to_vec(event)?;
        let key = event
            .get("order_id")
            .and_then(Value::as_str)
            .unwrap_or("inventory");

        tracing::info!(topic = %topic, key, "publishing inventory event");

        self.producer
            .send(
                FutureRecord::to(topic).key(key).payload(&payload),
                Duration::from_secs(5),
            )
            .await
            .map_err(|(err, _)| anyhow::anyhow!("failed to publish inventory event: {err}"))?;

        Ok(())
    }
}

#[async_trait]
impl InventoryEventPublisher for InventoryKafkaProducer {
    async fn publish_inventory_reserved(&self, event: &Value) -> anyhow::Result<()> {
        self.publish(&self.inventory_reserved_topic, event).await
    }

    async fn publish_inventory_released(&self, event: &Value) -> anyhow::Result<()> {
        self.publish(&self.inventory_released_topic, event).await
    }
}
