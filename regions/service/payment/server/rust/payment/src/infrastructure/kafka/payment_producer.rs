use crate::infrastructure::config::KafkaConfig;
use crate::proto::k1s0::event::service::payment::v1::{
    PaymentCompletedEvent, PaymentFailedEvent, PaymentInitiatedEvent, PaymentRefundedEvent,
};
use crate::usecase::event_publisher::PaymentEventPublisher;
use async_trait::async_trait;
use prost::Message;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;

// Kafka に Protobuf エンコードされた決済イベントを送信するプロデューサー
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
        // 冪等プロデューサーを有効化し、メッセージの重複送信を防止する
        client_config.set("enable.idempotence", "true");

        let producer = client_config.create()?;

        Ok(Self {
            producer,
            payment_initiated_topic: config.payment_initiated_topic.clone(),
            payment_completed_topic: config.payment_completed_topic.clone(),
            payment_failed_topic: config.payment_failed_topic.clone(),
            payment_refunded_topic: config.payment_refunded_topic.clone(),
        })
    }

    // Protobuf エンコード済みペイロードを指定トピックに送信する
    async fn publish(&self, topic: &str, key: &str, payload: &[u8]) -> anyhow::Result<()> {
        tracing::info!(topic = %topic, key, "publishing payment event");

        self.producer
            .send(
                FutureRecord::to(topic).key(key).payload(payload),
                Duration::from_secs(5),
            )
            .await
            .map_err(|(err, _)| anyhow::anyhow!("failed to publish payment event: {err}"))?;

        Ok(())
    }
}

#[async_trait]
impl PaymentEventPublisher for PaymentKafkaProducer {
    // 決済開始イベントを Protobuf エンコードして Kafka に publish する
    async fn publish_payment_initiated(&self, event: &PaymentInitiatedEvent) -> anyhow::Result<()> {
        let payload = event.encode_to_vec();
        self.publish(
            &self.payment_initiated_topic,
            event.payment_id.as_str(),
            &payload,
        )
        .await
    }

    // 決済完了イベントを Protobuf エンコードして Kafka に publish する
    async fn publish_payment_completed(&self, event: &PaymentCompletedEvent) -> anyhow::Result<()> {
        let payload = event.encode_to_vec();
        self.publish(
            &self.payment_completed_topic,
            event.payment_id.as_str(),
            &payload,
        )
        .await
    }

    // 決済失敗イベントを Protobuf エンコードして Kafka に publish する
    async fn publish_payment_failed(&self, event: &PaymentFailedEvent) -> anyhow::Result<()> {
        let payload = event.encode_to_vec();
        self.publish(
            &self.payment_failed_topic,
            event.payment_id.as_str(),
            &payload,
        )
        .await
    }

    // 返金イベントを Protobuf エンコードして Kafka に publish する
    async fn publish_payment_refunded(&self, event: &PaymentRefundedEvent) -> anyhow::Result<()> {
        let payload = event.encode_to_vec();
        self.publish(
            &self.payment_refunded_topic,
            event.payment_id.as_str(),
            &payload,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::k1s0::system::common::v1::EventMetadata;
    use prost::Message;

    #[test]
    fn test_payment_initiated_event_serialization() {
        // Protobuf シリアライズ・デシリアライズの往復検証
        let event = PaymentInitiatedEvent {
            metadata: Some(EventMetadata {
                event_id: "evt-001".to_string(),
                event_type: "payment.initiated".to_string(),
                source: "payment-server".to_string(),
                timestamp: 1700000000000,
                trace_id: "".to_string(),
                correlation_id: "pay-001".to_string(),
                schema_version: 1,
                // 因果関係IDは空文字列で初期化する
                causation_id: "".to_string(),
            }),
            payment_id: "pay-001".to_string(),
            order_id: "order-001".to_string(),
            customer_id: "cust-001".to_string(),
            amount: 5000,
            currency: "JPY".to_string(),
            payment_method: "credit_card".to_string(),
            initiated_at: None,
        };

        let bytes = event.encode_to_vec();
        assert!(!bytes.is_empty());

        let decoded = PaymentInitiatedEvent::decode(bytes.as_slice()).unwrap();
        assert_eq!(decoded.payment_id, "pay-001");
        assert_eq!(decoded.amount, 5000);
    }
}
