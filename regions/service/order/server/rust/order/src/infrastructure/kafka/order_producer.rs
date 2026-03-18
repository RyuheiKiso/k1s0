// Kafka を使った注文イベント publisher 実装。
// ドメインイベント型を受け取り、Proto型に変換してからProtobufシリアライズしてKafkaに送信する。

use crate::domain::entity::event::{
    OrderCancelledDomainEvent, OrderCreatedDomainEvent, OrderUpdatedDomainEvent,
};
use crate::infrastructure::config::KafkaConfig;
use crate::proto::k1s0::event::service::order::v1::{
    OrderCancelledEvent, OrderCreatedEvent, OrderItem as ProtoOrderItem, OrderUpdatedEvent,
};
use crate::proto::k1s0::system::common::v1::EventMetadata as ProtoEventMetadata;
use crate::usecase::event_publisher::OrderEventPublisher;
use async_trait::async_trait;
use prost::Message;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;

// Kafka を使った注文イベント publisher 実装
pub struct OrderKafkaProducer {
    producer: FutureProducer,
    order_created_topic: String,
    order_updated_topic: String,
    order_cancelled_topic: String,
}

impl OrderKafkaProducer {
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
            order_created_topic: config.order_created_topic.clone(),
            order_updated_topic: config.order_updated_topic.clone(),
            order_cancelled_topic: config.order_cancelled_topic.clone(),
        })
    }

    // Protobuf エンコード済みペイロードを指定トピックに publish する
    async fn publish(&self, topic: &str, key: &str, payload: &[u8]) -> anyhow::Result<()> {
        tracing::info!(topic = %topic, key, "publishing order event (protobuf)");

        self.producer
            .send(
                FutureRecord::to(topic).key(key).payload(payload),
                Duration::from_secs(5),
            )
            .await
            .map_err(|(err, _)| anyhow::anyhow!("failed to publish order event: {err}"))?;

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

#[async_trait]
impl OrderEventPublisher for OrderKafkaProducer {
    // 注文作成ドメインイベントをProto型に変換し、Protobuf シリアライズして Kafka に publish する
    async fn publish_order_created(&self, event: &OrderCreatedDomainEvent) -> anyhow::Result<()> {
        let proto_event = OrderCreatedEvent {
            metadata: convert_metadata(&event.metadata),
            order_id: event.order_id.clone(),
            customer_id: event.customer_id.clone(),
            items: event
                .items
                .iter()
                .map(|i| ProtoOrderItem {
                    product_id: i.product_id.clone(),
                    quantity: i.quantity,
                    unit_price: i.unit_price,
                })
                .collect(),
            total_amount: event.total_amount,
            currency: event.currency.clone(),
        };
        let payload = proto_event.encode_to_vec();
        self.publish(&self.order_created_topic, &event.order_id, &payload)
            .await
    }

    // 注文更新ドメインイベントをProto型に変換し、Protobuf シリアライズして Kafka に publish する
    async fn publish_order_updated(&self, event: &OrderUpdatedDomainEvent) -> anyhow::Result<()> {
        let proto_event = OrderUpdatedEvent {
            metadata: convert_metadata(&event.metadata),
            order_id: event.order_id.clone(),
            user_id: event.user_id.clone(),
            items: event
                .items
                .iter()
                .map(|i| ProtoOrderItem {
                    product_id: i.product_id.clone(),
                    quantity: i.quantity,
                    unit_price: i.unit_price,
                })
                .collect(),
            total_amount: event.total_amount,
            status: event.status.clone(),
        };
        let payload = proto_event.encode_to_vec();
        self.publish(&self.order_updated_topic, &event.order_id, &payload)
            .await
    }

    // 注文キャンセルドメインイベントをProto型に変換し、Protobuf シリアライズして Kafka に publish する
    async fn publish_order_cancelled(
        &self,
        event: &OrderCancelledDomainEvent,
    ) -> anyhow::Result<()> {
        let proto_event = OrderCancelledEvent {
            metadata: convert_metadata(&event.metadata),
            order_id: event.order_id.clone(),
            user_id: event.user_id.clone(),
            reason: event.reason.clone(),
        };
        let payload = proto_event.encode_to_vec();
        self.publish(&self.order_cancelled_topic, &event.order_id, &payload)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::event::{EventMetadata, OrderItemEvent};
    use prost::Message;

    #[test]
    fn test_order_created_event_serialization() {
        // ドメインイベントからProto型への変換とProtobuf往復検証
        let domain_event = OrderCreatedDomainEvent {
            metadata: Some(EventMetadata {
                event_id: "evt-001".to_string(),
                event_type: "order.created".to_string(),
                source: "order-server".to_string(),
                timestamp: 1700000000000,
                trace_id: "".to_string(),
                correlation_id: "order-001".to_string(),
                schema_version: 1,
                causation_id: "".to_string(),
            }),
            order_id: "order-001".to_string(),
            customer_id: "cust-001".to_string(),
            items: vec![OrderItemEvent {
                product_id: "prod-001".to_string(),
                quantity: 2,
                unit_price: 1000,
            }],
            total_amount: 2000,
            currency: "JPY".to_string(),
        };

        // ドメインイベントからProto型に変換
        let proto_event = OrderCreatedEvent {
            metadata: convert_metadata(&domain_event.metadata),
            order_id: domain_event.order_id.clone(),
            customer_id: domain_event.customer_id.clone(),
            items: domain_event
                .items
                .iter()
                .map(|i| ProtoOrderItem {
                    product_id: i.product_id.clone(),
                    quantity: i.quantity,
                    unit_price: i.unit_price,
                })
                .collect(),
            total_amount: domain_event.total_amount,
            currency: domain_event.currency.clone(),
        };

        let bytes = proto_event.encode_to_vec();
        assert!(!bytes.is_empty());

        // デシリアライズして元のフィールド値と一致することを確認
        let decoded = OrderCreatedEvent::decode(bytes.as_slice()).unwrap();
        assert_eq!(decoded.order_id, "order-001");
        assert_eq!(decoded.items.len(), 1);
        assert_eq!(decoded.items[0].quantity, 2);
    }
}
