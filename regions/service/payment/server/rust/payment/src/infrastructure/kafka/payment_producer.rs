// Kafka を使った決済イベント publisher 実装。
// ドメインイベント型を受け取り、Proto型に変換してからProtobufシリアライズしてKafkaに送信する。

use crate::domain::entity::event::{
    PaymentCompletedDomainEvent, PaymentFailedDomainEvent, PaymentInitiatedDomainEvent,
    PaymentRefundedDomainEvent,
};
use crate::infrastructure::config::KafkaConfig;
use crate::proto::k1s0::event::service::payment::v1::{
    PaymentCompletedEvent, PaymentFailedEvent, PaymentInitiatedEvent, PaymentRefundedEvent,
};
use crate::proto::k1s0::system::common::v1::{
    EventMetadata as ProtoEventMetadata, Timestamp as ProtoTimestamp,
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
impl PaymentEventPublisher for PaymentKafkaProducer {
    // 決済開始ドメインイベントをProto型に変換し、Protobuf エンコードして Kafka に publish する
    async fn publish_payment_initiated(
        &self,
        event: &PaymentInitiatedDomainEvent,
    ) -> anyhow::Result<()> {
        let proto_event = PaymentInitiatedEvent {
            metadata: convert_metadata(&event.metadata),
            payment_id: event.payment_id.clone(),
            order_id: event.order_id.clone(),
            customer_id: event.customer_id.clone(),
            amount: event.amount,
            currency: event.currency.clone(),
            payment_method: event.payment_method.clone(),
            initiated_at: convert_timestamp(&event.initiated_at),
        };
        let payload = proto_event.encode_to_vec();
        self.publish(
            &self.payment_initiated_topic,
            event.payment_id.as_str(),
            &payload,
        )
        .await
    }

    // 決済完了ドメインイベントをProto型に変換し、Protobuf エンコードして Kafka に publish する
    async fn publish_payment_completed(
        &self,
        event: &PaymentCompletedDomainEvent,
    ) -> anyhow::Result<()> {
        let proto_event = PaymentCompletedEvent {
            metadata: convert_metadata(&event.metadata),
            payment_id: event.payment_id.clone(),
            order_id: event.order_id.clone(),
            amount: event.amount,
            currency: event.currency.clone(),
            transaction_id: event.transaction_id.clone(),
            completed_at: convert_timestamp(&event.completed_at),
        };
        let payload = proto_event.encode_to_vec();
        self.publish(
            &self.payment_completed_topic,
            event.payment_id.as_str(),
            &payload,
        )
        .await
    }

    // 決済失敗ドメインイベントをProto型に変換し、Protobuf エンコードして Kafka に publish する
    async fn publish_payment_failed(&self, event: &PaymentFailedDomainEvent) -> anyhow::Result<()> {
        let proto_event = PaymentFailedEvent {
            metadata: convert_metadata(&event.metadata),
            payment_id: event.payment_id.clone(),
            order_id: event.order_id.clone(),
            reason: event.reason.clone(),
            error_code: event.error_code.clone(),
            failed_at: convert_timestamp(&event.failed_at),
        };
        let payload = proto_event.encode_to_vec();
        self.publish(
            &self.payment_failed_topic,
            event.payment_id.as_str(),
            &payload,
        )
        .await
    }

    // 返金ドメインイベントをProto型に変換し、Protobuf エンコードして Kafka に publish する
    async fn publish_payment_refunded(
        &self,
        event: &PaymentRefundedDomainEvent,
    ) -> anyhow::Result<()> {
        let proto_event = PaymentRefundedEvent {
            metadata: convert_metadata(&event.metadata),
            payment_id: event.payment_id.clone(),
            order_id: event.order_id.clone(),
            refund_amount: event.refund_amount,
            currency: event.currency.clone(),
            reason: event.reason.clone(),
            refunded_at: convert_timestamp(&event.refunded_at),
        };
        let payload = proto_event.encode_to_vec();
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
    use crate::domain::entity::event::EventMetadata;
    use prost::Message;

    #[test]
    fn test_payment_initiated_event_serialization() {
        // ドメインイベントからProto型への変換とProtobuf往復検証
        let domain_event = PaymentInitiatedDomainEvent {
            metadata: Some(EventMetadata {
                event_id: "evt-001".to_string(),
                event_type: "payment.initiated".to_string(),
                source: "payment-server".to_string(),
                timestamp: 1700000000000,
                trace_id: "".to_string(),
                correlation_id: "pay-001".to_string(),
                schema_version: 1,
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

        // ドメインイベントからProto型に変換
        let proto_event = PaymentInitiatedEvent {
            metadata: convert_metadata(&domain_event.metadata),
            payment_id: domain_event.payment_id.clone(),
            order_id: domain_event.order_id.clone(),
            customer_id: domain_event.customer_id.clone(),
            amount: domain_event.amount,
            currency: domain_event.currency.clone(),
            payment_method: domain_event.payment_method.clone(),
            initiated_at: convert_timestamp(&domain_event.initiated_at),
        };

        let bytes = proto_event.encode_to_vec();
        assert!(!bytes.is_empty());

        let decoded = PaymentInitiatedEvent::decode(bytes.as_slice()).unwrap();
        assert_eq!(decoded.payment_id, "pay-001");
        assert_eq!(decoded.amount, 5000);
    }
}
