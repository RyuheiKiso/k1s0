use crate::infrastructure::config::KafkaConfig;
use crate::proto::k1s0::event::service::order::v1::{
    OrderCancelledEvent, OrderCreatedEvent, OrderUpdatedEvent,
};
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

#[async_trait]
impl OrderEventPublisher for OrderKafkaProducer {
    // 注文作成イベントを Protobuf シリアライズして Kafka に publish する
    async fn publish_order_created(&self, event: &OrderCreatedEvent) -> anyhow::Result<()> {
        let payload = event.encode_to_vec();
        self.publish(&self.order_created_topic, &event.order_id, &payload)
            .await
    }

    // 注文更新イベントを Protobuf シリアライズして Kafka に publish する
    async fn publish_order_updated(&self, event: &OrderUpdatedEvent) -> anyhow::Result<()> {
        let payload = event.encode_to_vec();
        self.publish(&self.order_updated_topic, &event.order_id, &payload)
            .await
    }

    // 注文キャンセルイベントを Protobuf シリアライズして Kafka に publish する
    async fn publish_order_cancelled(&self, event: &OrderCancelledEvent) -> anyhow::Result<()> {
        let payload = event.encode_to_vec();
        self.publish(&self.order_cancelled_topic, &event.order_id, &payload)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::k1s0::event::service::order::v1::OrderItem;
    use crate::proto::k1s0::system::common::v1::EventMetadata;
    use prost::Message;

    #[test]
    fn test_order_created_event_serialization() {
        // Protobuf シリアライズ・デシリアライズの往復検証
        let event = OrderCreatedEvent {
            metadata: Some(EventMetadata {
                event_id: "evt-001".to_string(),
                event_type: "order.created".to_string(),
                source: "order-server".to_string(),
                timestamp: 1700000000000,
                trace_id: "".to_string(),
                correlation_id: "order-001".to_string(),
                schema_version: 1,
            }),
            order_id: "order-001".to_string(),
            customer_id: "cust-001".to_string(),
            items: vec![OrderItem {
                product_id: "prod-001".to_string(),
                quantity: 2,
                unit_price: 1000,
            }],
            total_amount: 2000,
            currency: "JPY".to_string(),
        };

        let bytes = event.encode_to_vec();
        assert!(!bytes.is_empty());

        // デシリアライズして元のフィールド値と一致することを確認
        let decoded = OrderCreatedEvent::decode(bytes.as_slice()).unwrap();
        assert_eq!(decoded.order_id, "order-001");
        assert_eq!(decoded.items.len(), 1);
        assert_eq!(decoded.items[0].quantity, 2);
    }
}
