use crate::domain::entity::payment::{Payment, PaymentStatus};
use crate::domain::error::PaymentError;
use crate::domain::repository::payment_repository::PaymentRepository;
use crate::domain::service::payment_service::PaymentDomainService;
use std::sync::Arc;
use uuid::Uuid;

pub struct RefundPaymentUseCase {
    payment_repo: Arc<dyn PaymentRepository>,
}

impl RefundPaymentUseCase {
    pub fn new(payment_repo: Arc<dyn PaymentRepository>) -> Self {
        Self { payment_repo }
    }

    pub async fn execute(&self, payment_id: Uuid) -> anyhow::Result<Payment> {
        let existing = self
            .payment_repo
            .find_by_id(payment_id)
            .await?
            .ok_or_else(|| PaymentError::NotFound(payment_id.to_string()))?;

        // ステータス遷移バリデーション
        PaymentDomainService::validate_status_transition(
            &existing.status,
            &PaymentStatus::Refunded,
        )?;

        // ステータス更新（Outbox イベントも同一トランザクション内で挿入される）
        let updated = self
            .payment_repo
            .refund(payment_id, existing.version)
            .await?;

        Ok(updated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::payment_repository::MockPaymentRepository;
    use chrono::Utc;

    fn sample_payment(status: PaymentStatus) -> Payment {
        Payment {
            id: Uuid::new_v4(),
            order_id: "ORD-001".to_string(),
            customer_id: "CUST-001".to_string(),
            amount: 5000,
            currency: "JPY".to_string(),
            status,
            payment_method: Some("credit_card".to_string()),
            transaction_id: Some("TXN-001".to_string()),
            error_code: None,
            error_message: None,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_refund_payment_success() {
        let payment = sample_payment(PaymentStatus::Completed);
        let payment_id = payment.id;
        let mut refunded_payment = payment.clone();
        refunded_payment.status = PaymentStatus::Refunded;
        let payment_clone = payment.clone();
        let refunded_clone = refunded_payment.clone();

        let mut mock_repo = MockPaymentRepository::new();

        mock_repo
            .expect_find_by_id()
            .withf(move |id| *id == payment_id)
            .times(1)
            .returning(move |_| Ok(Some(payment_clone.clone())));

        mock_repo
            .expect_refund()
            .times(1)
            .returning(move |_, _| Ok(refunded_clone.clone()));

        let uc = RefundPaymentUseCase::new(Arc::new(mock_repo));
        let result = uc.execute(payment_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, PaymentStatus::Refunded);
    }

    #[tokio::test]
    async fn test_refund_payment_invalid_transition() {
        let payment = sample_payment(PaymentStatus::Initiated);
        let payment_id = payment.id;
        let payment_clone = payment.clone();

        let mut mock_repo = MockPaymentRepository::new();

        mock_repo
            .expect_find_by_id()
            .times(1)
            .returning(move |_| Ok(Some(payment_clone.clone())));

        let uc = RefundPaymentUseCase::new(Arc::new(mock_repo));
        let result = uc.execute(payment_id).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("invalid status transition"));
    }
}
