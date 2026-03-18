// Kafka を使った在庫イベント publisher 実装。
// ドメインイベント型を受け取り、Proto型に変換してからProtobufシリアライズしてKafkaに送信する。

use crate::domain::entity::event::{InventoryReleasedDomainEvent, InventoryReservedDomainEvent};
use crate::infrastructure::config::KafkaConfig;
use crate::proto::k1s0::event::service::inventory::v1::{
    InventoryReleasedEvent, InventoryReservedEvent,
};
use crate::proto::k1s0::system::common::v1::{
    EventMetadata as ProtoEventMetadata, Timestamp as ProtoTimestamp,
};
use crate::usecase::event_publisher::InventoryEventPublisher;
use async_trait::async_trait;
use prost::Message;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
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
        // 冪等プロデューサーを有効化し、メッセージの重複送信を防止する
        client_config.set("enable.idempotence", "true");

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

// ドメインイベントメタデータからProto型メタデータへの変換
fn convert_metadata(
    metadata: &Option<crate::domain::entity::event::EventMetadata>,
) -> Option<ProtoEventMetadata> {
    metadata.as_ref().map(|m| ProtoEventMetadata {
        event_id: m.event_id.clone(),
        event_type: m.event_type.clone(),
        source: m.source.clone(),
        timestamp: m.timestamp,
        trace_id: m.trace_id.clone(),
        correlation_id: m.correlation_id.clone(),
        schema_version: m.schema_version,
        causation_id: m.causation_id.clone(),
    })
}

// chrono::DateTime を Proto Timestamp に変換する
fn convert_timestamp(dt: &Option<chrono::DateTime<chrono::Utc>>) -> Option<ProtoTimestamp> {
    dt.as_ref().map(|t| ProtoTimestamp {
        seconds: t.timestamp(),
        nanos: t.timestamp_subsec_nanos() as i32,
    })
}

#[async_trait]
impl InventoryEventPublisher for InventoryKafkaProducer {
    // 在庫予約ドメインイベントをProto型に変換し、Protobuf シリアライズして Kafka に publish する
    async fn publish_inventory_reserved(
        &self,
        event: &InventoryReservedDomainEvent,
    ) -> anyhow::Result<()> {
        let proto_event = InventoryReservedEvent {
            metadata: convert_metadata(&event.metadata),
            order_id: event.order_id.clone(),
            product_id: event.product_id.clone(),
            quantity: event.quantity,
            warehouse_id: event.warehouse_id.clone(),
            reserved_at: convert_timestamp(&event.reserved_at),
        };
        let payload = proto_event.encode_to_vec();
        let key = event.order_id.as_str();
        self.publish(&self.inventory_reserved_topic, key, &payload)
            .await
    }

    // 在庫解放ドメインイベントをProto型に変換し、Protobuf シリアライズして Kafka に publish する
    async fn publish_inventory_released(
        &self,
        event: &InventoryReleasedDomainEvent,
    ) -> anyhow::Result<()> {
        let proto_event = InventoryReleasedEvent {
            metadata: convert_metadata(&event.metadata),
            order_id: event.order_id.clone(),
            product_id: event.product_id.clone(),
            quantity: event.quantity,
            warehouse_id: event.warehouse_id.clone(),
            reason: event.reason.clone(),
            released_at: convert_timestamp(&event.released_at),
        };
        let payload = proto_event.encode_to_vec();
        let key = event.order_id.as_str();
        self.publish(&self.inventory_released_topic, key, &payload)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::event::EventMetadata;
    use prost::Message;

    #[test]
    fn test_inventory_reserved_event_serialization() {
        // ドメインイベントからProto型への変換とProtobuf往復検証
        let domain_event = InventoryReservedDomainEvent {
            metadata: Some(EventMetadata {
                event_id: "evt-001".to_string(),
                event_type: "inventory.reserved".to_string(),
                source: "inventory-server".to_string(),
                timestamp: 1700000000000,
                trace_id: "".to_string(),
                correlation_id: "order-001".to_string(),
                schema_version: 1,
                causation_id: "".to_string(),
            }),
            order_id: "order-001".to_string(),
            product_id: "prod-001".to_string(),
            quantity: 5,
            warehouse_id: "wh-001".to_string(),
            reserved_at: None,
        };

        // ドメインイベントからProto型に変換
        let proto_event = InventoryReservedEvent {
            metadata: convert_metadata(&domain_event.metadata),
            order_id: domain_event.order_id.clone(),
            product_id: domain_event.product_id.clone(),
            quantity: domain_event.quantity,
            warehouse_id: domain_event.warehouse_id.clone(),
            reserved_at: convert_timestamp(&domain_event.reserved_at),
        };

        let bytes = proto_event.encode_to_vec();
        assert!(!bytes.is_empty());

        let decoded = InventoryReservedEvent::decode(bytes.as_slice()).unwrap();
        assert_eq!(decoded.order_id, "order-001");
        assert_eq!(decoded.quantity, 5);
    }
}
