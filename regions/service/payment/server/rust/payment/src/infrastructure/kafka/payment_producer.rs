use crate::infrastructure::config::KafkaConfig;
use crate::usecase::event_publisher::PaymentEventPublisher;
use async_trait::async_trait;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde_json::Value;
use std::time::Duration;

// TODO(NCR-007): Protobuf シリアライズ完全移行
// 現在は outbox JSONB -> JSON publish だが、
// api/proto/k1s0/event/service/payment/v1/payment_events.proto の
// Rust 生成型を使って prost::Message::encode_to_vec() でシリアライズすべき。
// build.rs で tonic_build を設定し、生成コードを src/proto/ に出力する。

pub struct PaymentKafkaProducer {
    producer: FutureProducer,
    payment_initiated_topic: String,
    payment_completed_topic: String,
    payment_failed_topic: String,
    payment_refunded_topic: String,
}

impl PaymentKafkaProducer {
    pub fn new(config: &KafkaConfig) -> anyhow::Result<Self> {
        let mut client_config = ClientConfig::new();
        client_config.set("bootstrap.servers", config.brokers.join(","));
        client_config.set("security.protocol", &config.security_protocol);
        client_config.set("acks", "all");
        client_config.set("message.timeout.ms", "5000");

        let producer = client_config.create()?;

        Ok(Self {
            producer,
            payment_initiated_topic: config.payment_initiated_topic.clone(),
            payment_completed_topic: config.payment_completed_topic.clone(),
            payment_failed_topic: config.payment_failed_topic.clone(),
            payment_refunded_topic: config.payment_refunded_topic.clone(),
        })
    }

    async fn publish(&self, topic: &str, event: &Value) -> anyhow::Result<()> {
        let payload = serde_json::to_vec(event)?;
        let key = event
            .get("payment_id")
            .and_then(Value::as_str)
            .unwrap_or("payment");

        tracing::info!(topic = %topic, key, "publishing payment event");

        self.producer
            .send(
                FutureRecord::to(topic).key(key).payload(&payload),
                Duration::from_secs(5),
            )
            .await
            .map_err(|(err, _)| anyhow::anyhow!("failed to publish payment event: {err}"))?;

        Ok(())
    }
}

#[async_trait]
impl PaymentEventPublisher for PaymentKafkaProducer {
    async fn publish_payment_initiated(&self, event: &Value) -> anyhow::Result<()> {
        self.publish(&self.payment_initiated_topic, event).await
    }

    async fn publish_payment_completed(&self, event: &Value) -> anyhow::Result<()> {
        self.publish(&self.payment_completed_topic, event).await
    }

    async fn publish_payment_failed(&self, event: &Value) -> anyhow::Result<()> {
        self.publish(&self.payment_failed_topic, event).await
    }

    async fn publish_payment_refunded(&self, event: &Value) -> anyhow::Result<()> {
        self.publish(&self.payment_refunded_topic, event).await
    }
}
