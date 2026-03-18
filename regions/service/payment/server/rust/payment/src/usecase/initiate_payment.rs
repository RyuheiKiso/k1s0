use crate::domain::entity::payment::{InitiatePayment, Payment};
use crate::domain::repository::payment_repository::PaymentRepository;
use crate::domain::service::payment_service::PaymentDomainService;
use std::sync::Arc;

pub struct InitiatePaymentUseCase {
    payment_repo: Arc<dyn PaymentRepository>,
}

impl InitiatePaymentUseCase {
    pub fn new(payment_repo: Arc<dyn PaymentRepository>) -> Self {
        Self { payment_repo }
    }

    pub async fn execute(&self, input: &InitiatePayment) -> anyhow::Result<Payment> {
        // ドメインバリデーション（PaymentError を返す）
        PaymentDomainService::validate_initiate_payment(input)?;

        // 冪等性チェック: 同一 order_id の決済が既に存在する場合はそれを返す（二重課金防止）
        if let Some(existing) = self.payment_repo.find_by_order_id(&input.order_id).await? {
            tracing::info!(
                order_id = %input.order_id,
                payment_id = %existing.id,
                "idempotent payment request detected, returning existing payment"
            );
            return Ok(existing);
        }

        // 永続化（Outbox イベントも同一トランザクション内で挿入される）
        let payment = self.payment_repo.create(input).await?;

        Ok(payment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::payment::PaymentStatus;
    use crate::domain::repository::payment_repository::MockPaymentRepository;
    use chrono::Utc;
    use uuid::Uuid;

    fn sample_initiate_payment() -> InitiatePayment {
        InitiatePayment {
            order_id: "ORD-001".to_string(),
            customer_id: "CUST-001".to_string(),
            amount: 5000,
            currency: "JPY".to_string(),
            payment_method: Some("credit_card".to_string()),
        }
    }

    fn sample_payment() -> Payment {
        Payment {
            id: Uuid::new_v4(),
            order_id: "ORD-001".to_string(),
            customer_id: "CUST-001".to_string(),
            amount: 5000,
            currency: "JPY".to_string(),
            status: PaymentStatus::Initiated,
            payment_method: Some("credit_card".to_string()),
            transaction_id: None,
            error_code: None,
            error_message: None,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_initiate_payment_success() {
        let payment = sample_payment();
        let payment_clone = payment.clone();

        let mut mock_repo = MockPaymentRepository::new();

        mock_repo
            .expect_create()
            .times(1)
            .returning(move |_| Ok(payment_clone.clone()));

        let uc = InitiatePaymentUseCase::new(Arc::new(mock_repo));
        let input = sample_initiate_payment();
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let created_payment = result.unwrap();
        assert_eq!(created_payment.order_id, "ORD-001");
        assert_eq!(created_payment.status, PaymentStatus::Initiated);
    }

    #[tokio::test]
    async fn test_initiate_payment_validation_failure() {
        let mock_repo = MockPaymentRepository::new();

        let uc = InitiatePaymentUseCase::new(Arc::new(mock_repo));
        let input = InitiatePayment {
            order_id: "".to_string(),
            customer_id: "CUST-001".to_string(),
            amount: 5000,
            currency: "JPY".to_string(),
            payment_method: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
    }
}
