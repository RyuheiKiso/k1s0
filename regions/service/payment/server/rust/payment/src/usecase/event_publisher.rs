use async_trait::async_trait;
use serde_json::Value;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait PaymentEventPublisher: Send + Sync {
    async fn publish_payment_initiated(&self, event: &Value) -> anyhow::Result<()>;
    async fn publish_payment_completed(&self, event: &Value) -> anyhow::Result<()>;
    async fn publish_payment_failed(&self, event: &Value) -> anyhow::Result<()>;
    async fn publish_payment_refunded(&self, event: &Value) -> anyhow::Result<()>;
}

pub struct NoopPaymentEventPublisher;

#[async_trait]
impl PaymentEventPublisher for NoopPaymentEventPublisher {
    async fn publish_payment_initiated(&self, _event: &Value) -> anyhow::Result<()> {
        Ok(())
    }

    async fn publish_payment_completed(&self, _event: &Value) -> anyhow::Result<()> {
        Ok(())
    }

    async fn publish_payment_failed(&self, _event: &Value) -> anyhow::Result<()> {
        Ok(())
    }

    async fn publish_payment_refunded(&self, _event: &Value) -> anyhow::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_noop_publisher_payment_initiated() {
        let publisher = NoopPaymentEventPublisher;
        let event = serde_json::json!({"event_type": "payment.initiated"});
        assert!(publisher.publish_payment_initiated(&event).await.is_ok());
    }

    #[tokio::test]
    async fn test_noop_publisher_payment_completed() {
        let publisher = NoopPaymentEventPublisher;
        let event = serde_json::json!({"event_type": "payment.completed"});
        assert!(publisher.publish_payment_completed(&event).await.is_ok());
    }

    #[tokio::test]
    async fn test_noop_publisher_payment_failed() {
        let publisher = NoopPaymentEventPublisher;
        let event = serde_json::json!({"event_type": "payment.failed"});
        assert!(publisher.publish_payment_failed(&event).await.is_ok());
    }

    #[tokio::test]
    async fn test_noop_publisher_payment_refunded() {
        let publisher = NoopPaymentEventPublisher;
        let event = serde_json::json!({"event_type": "payment.refunded"});
        assert!(publisher.publish_payment_refunded(&event).await.is_ok());
    }
}
