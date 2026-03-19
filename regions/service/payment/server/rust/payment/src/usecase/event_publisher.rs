// 決済イベントパブリッシャートレイト。
// usecase層はドメインイベント型のみに依存し、Proto型には依存しない。
// Proto型への変換はインフラ層（Kafka producer）で行う。

use crate::domain::entity::event::{
    PaymentCompletedDomainEvent, PaymentFailedDomainEvent, PaymentInitiatedDomainEvent,
    PaymentRefundedDomainEvent,
};
use async_trait::async_trait;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait PaymentEventPublisher: Send + Sync {
    // 決済開始イベントをドメインイベント型で publish する
    async fn publish_payment_initiated(
        &self,
        event: &PaymentInitiatedDomainEvent,
    ) -> anyhow::Result<()>;
    // 決済完了イベントをドメインイベント型で publish する
    async fn publish_payment_completed(
        &self,
        event: &PaymentCompletedDomainEvent,
    ) -> anyhow::Result<()>;
    // 決済失敗イベントをドメインイベント型で publish する
    async fn publish_payment_failed(&self, event: &PaymentFailedDomainEvent) -> anyhow::Result<()>;
    // 返金イベントをドメインイベント型で publish する
    async fn publish_payment_refunded(
        &self,
        event: &PaymentRefundedDomainEvent,
    ) -> anyhow::Result<()>;
}

// イベントを何も行わずに成功を返す Noop 実装（テスト・開発用）
pub struct NoopPaymentEventPublisher;

#[async_trait]
impl PaymentEventPublisher for NoopPaymentEventPublisher {
    async fn publish_payment_initiated(
        &self,
        _event: &PaymentInitiatedDomainEvent,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn publish_payment_completed(
        &self,
        _event: &PaymentCompletedDomainEvent,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn publish_payment_failed(
        &self,
        _event: &PaymentFailedDomainEvent,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn publish_payment_refunded(
        &self,
        _event: &PaymentRefundedDomainEvent,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::event::EventMetadata;

    #[tokio::test]
    async fn test_noop_publisher_payment_initiated() {
        let publisher = NoopPaymentEventPublisher;
        let event = PaymentInitiatedDomainEvent {
            metadata: Some(EventMetadata {
                event_id: "evt-001".to_string(),
                event_type: "payment.initiated".to_string(),
                source: "test".to_string(),
                timestamp: 0,
                trace_id: "".to_string(),
                correlation_id: "".to_string(),
                schema_version: 1,
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
        let event = PaymentCompletedDomainEvent {
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
        let event = PaymentFailedDomainEvent {
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
        let event = PaymentRefundedDomainEvent {
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
