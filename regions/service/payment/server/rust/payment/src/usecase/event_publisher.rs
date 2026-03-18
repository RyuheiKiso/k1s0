use crate::proto::k1s0::event::service::payment::v1::{
    PaymentCompletedEvent, PaymentFailedEvent, PaymentInitiatedEvent, PaymentRefundedEvent,
};
use async_trait::async_trait;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait PaymentEventPublisher: Send + Sync {
    // 決済開始イベントを Protobuf 形式で Kafka に publish する
    async fn publish_payment_initiated(&self, event: &PaymentInitiatedEvent) -> anyhow::Result<()>;
    // 決済完了イベントを Protobuf 形式で Kafka に publish する
    async fn publish_payment_completed(&self, event: &PaymentCompletedEvent) -> anyhow::Result<()>;
    // 決済失敗イベントを Protobuf 形式で Kafka に publish する
    async fn publish_payment_failed(&self, event: &PaymentFailedEvent) -> anyhow::Result<()>;
    // 返金イベントを Protobuf 形式で Kafka に publish する
    async fn publish_payment_refunded(&self, event: &PaymentRefundedEvent) -> anyhow::Result<()>;
}

// イベントを何も行わずに成功を返す Noop 実装（テスト・開発用）
pub struct NoopPaymentEventPublisher;

#[async_trait]
impl PaymentEventPublisher for NoopPaymentEventPublisher {
    async fn publish_payment_initiated(
        &self,
        _event: &PaymentInitiatedEvent,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn publish_payment_completed(
        &self,
        _event: &PaymentCompletedEvent,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn publish_payment_failed(&self, _event: &PaymentFailedEvent) -> anyhow::Result<()> {
        Ok(())
    }

    async fn publish_payment_refunded(&self, _event: &PaymentRefundedEvent) -> anyhow::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::k1s0::system::common::v1::EventMetadata;

    #[tokio::test]
    async fn test_noop_publisher_payment_initiated() {
        let publisher = NoopPaymentEventPublisher;
        let event = PaymentInitiatedEvent {
            metadata: Some(EventMetadata {
                event_id: "evt-001".to_string(),
                event_type: "payment.initiated".to_string(),
                source: "test".to_string(),
                timestamp: 0,
                trace_id: "".to_string(),
                correlation_id: "".to_string(),
                schema_version: 1,
                // 因果関係IDは空文字列で初期化する
                causation_id: "".to_string(),
            }),
            payment_id: "pay-001".to_string(),
            order_id: "order-001".to_string(),
            customer_id: "cust-001".to_string(),
            amount: 1000,
            currency: "JPY".to_string(),
            payment_method: "credit_card".to_string(),
            initiated_at: None,
        };
        assert!(publisher.publish_payment_initiated(&event).await.is_ok());
    }

    #[tokio::test]
    async fn test_noop_publisher_payment_completed() {
        let publisher = NoopPaymentEventPublisher;
        let event = PaymentCompletedEvent {
            metadata: None,
            payment_id: "pay-001".to_string(),
            order_id: "order-001".to_string(),
            amount: 1000,
            currency: "JPY".to_string(),
            transaction_id: "txn-001".to_string(),
            completed_at: None,
        };
        assert!(publisher.publish_payment_completed(&event).await.is_ok());
    }

    #[tokio::test]
    async fn test_noop_publisher_payment_failed() {
        let publisher = NoopPaymentEventPublisher;
        let event = PaymentFailedEvent {
            metadata: None,
            payment_id: "pay-001".to_string(),
            order_id: "order-001".to_string(),
            reason: "insufficient_funds".to_string(),
            error_code: "E001".to_string(),
            failed_at: None,
        };
        assert!(publisher.publish_payment_failed(&event).await.is_ok());
    }

    #[tokio::test]
    async fn test_noop_publisher_payment_refunded() {
        let publisher = NoopPaymentEventPublisher;
        let event = PaymentRefundedEvent {
            metadata: None,
            payment_id: "pay-001".to_string(),
            order_id: "order-001".to_string(),
            refund_amount: 500,
            currency: "JPY".to_string(),
            reason: "customer_request".to_string(),
            refunded_at: None,
        };
        assert!(publisher.publish_payment_refunded(&event).await.is_ok());
    }
}
