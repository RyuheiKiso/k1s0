use crate::infrastructure::config::KafkaConfig;
use crate::usecase::event_publisher::InventoryEventPublisher;
use async_trait::async_trait;
use prost::Message;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;

use crate::proto::k1s0::event::service::inventory::v1::{
    InventoryReservedEvent, InventoryReleasedEvent,
};

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

    // Protobuf エンコード済みバイト列を指定トピックに publish する
    async fn publish(&self, topic: &str, key: &str, payload: &[u8]) -> anyhow::Result<()> {
        tracing::info!(topic = %topic, key, "publishing inventory event");

        self.producer
            .send(
                FutureRecord::to(topic).key(key).payload(payload),
                Duration::from_secs(5),
            )
            .await
            .map_err(|(err, _)| anyhow::anyhow!("failed to publish inventory event: {err}"))?;

        Ok(())
    }
}

#[async_trait]
impl InventoryEventPublisher for InventoryKafkaProducer {
    // 在庫予約イベントを Protobuf シリアライズして Kafka に publish する
    async fn publish_inventory_reserved(&self, event: &InventoryReservedEvent) -> anyhow::Result<()> {
        let payload = event.encode_to_vec();
        let key = event.order_id.as_str();
        self.publish(&self.inventory_reserved_topic, key, &payload).await
    }

    // 在庫解放イベントを Protobuf シリアライズして Kafka に publish する
    async fn publish_inventory_released(&self, event: &InventoryReleasedEvent) -> anyhow::Result<()> {
        let payload = event.encode_to_vec();
        let key = event.order_id.as_str();
        self.publish(&self.inventory_released_topic, key, &payload).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prost::Message;
    use crate::proto::k1s0::system::common::v1::EventMetadata;

    #[test]
    fn test_inventory_reserved_event_serialization() {
        // Protobuf シリアライズ・デシリアライズの往復検証
        let event = InventoryReservedEvent {
            metadata: Some(EventMetadata {
                event_id: "evt-001".to_string(),
                event_type: "inventory.reserved".to_string(),
                source: "inventory-server".to_string(),
                timestamp: 1700000000000,
                trace_id: "".to_string(),
                correlation_id: "order-001".to_string(),
                schema_version: 1,
            }),
            order_id: "order-001".to_string(),
            product_id: "prod-001".to_string(),
            quantity: 5,
            warehouse_id: "wh-001".to_string(),
            reserved_at: None,
        };

        let bytes = event.encode_to_vec();
        assert!(!bytes.is_empty());

        let decoded = InventoryReservedEvent::decode(bytes.as_slice()).unwrap();
        assert_eq!(decoded.order_id, "order-001");
        assert_eq!(decoded.quantity, 5);
    }
}
