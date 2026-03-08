use crate::infrastructure::config::KafkaConfig;
use crate::usecase::event_publisher::OrderEventPublisher;
use async_trait::async_trait;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde_json::Value;
use std::time::Duration;

// TODO(NCR-006): Protobuf シリアライズ完全移行
// 現在は outbox JSONB → JSON publish だが、
// api/proto/k1s0/event/service/order/v1/order_events.proto の
// Rust 生成型を使って prost::Message::encode_to_vec() でシリアライズすべき。
// build.rs で tonic_build を設定し、生成コードを src/proto/ に出力する。

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

        let producer = client_config.create()?;

        Ok(Self {
            producer,
            order_created_topic: config.order_created_topic.clone(),
            order_updated_topic: config.order_updated_topic.clone(),
            order_cancelled_topic: config.order_cancelled_topic.clone(),
        })
    }

    async fn publish(&self, topic: &str, event: &Value) -> anyhow::Result<()> {
        let payload = serde_json::to_vec(event)?;
        let key = event
            .get("order_id")
            .and_then(Value::as_str)
            .unwrap_or("order");

        tracing::info!(topic = %topic, key, "publishing order event");

        self.producer
            .send(
                FutureRecord::to(topic).key(key).payload(&payload),
                Duration::from_secs(5),
            )
            .await
            .map_err(|(err, _)| anyhow::anyhow!("failed to publish order event: {err}"))?;

        Ok(())
    }
}

#[async_trait]
impl OrderEventPublisher for OrderKafkaProducer {
    async fn publish_order_created(&self, event: &Value) -> anyhow::Result<()> {
        self.publish(&self.order_created_topic, event).await
    }

    async fn publish_order_updated(&self, event: &Value) -> anyhow::Result<()> {
        self.publish(&self.order_updated_topic, event).await
    }

    async fn publish_order_cancelled(&self, event: &Value) -> anyhow::Result<()> {
        self.publish(&self.order_cancelled_topic, event).await
    }
}
