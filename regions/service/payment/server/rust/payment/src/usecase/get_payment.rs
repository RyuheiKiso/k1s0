use crate::domain::entity::payment::Payment;
use crate::domain::error::PaymentError;
use crate::domain::repository::payment_repository::PaymentRepository;
use std::sync::Arc;
use uuid::Uuid;

pub struct GetPaymentUseCase {
    payment_repo: Arc<dyn PaymentRepository>,
}

impl GetPaymentUseCase {
    pub fn new(payment_repo: Arc<dyn PaymentRepository>) -> Self {
        Self { payment_repo }
    }

    pub async fn execute(&self, payment_id: Uuid) -> anyhow::Result<Payment> {
        let payment = self
            .payment_repo
            .find_by_id(payment_id)
            .await?
            .ok_or_else(|| PaymentError::NotFound(payment_id.to_string()))?;

        Ok(payment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::payment::PaymentStatus;
    use crate::domain::repository::payment_repository::MockPaymentRepository;
    use chrono::Utc;

    #[tokio::test]
    async fn test_get_payment_success() {
        let payment_id = Uuid::new_v4();
        let payment = Payment {
            id: payment_id,
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
        };
        let payment_clone = payment.clone();

        let mut mock_repo = MockPaymentRepository::new();
        mock_repo
            .expect_find_by_id()
            .withf(move |id| *id == payment_id)
            .times(1)
            .returning(move |_| Ok(Some(payment_clone.clone())));

        let uc = GetPaymentUseCase::new(Arc::new(mock_repo));
        let result = uc.execute(payment_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, payment_id);
    }

    #[tokio::test]
    async fn test_get_payment_not_found() {
        let payment_id = Uuid::new_v4();
        let mut mock_repo = MockPaymentRepository::new();
        mock_repo
            .expect_find_by_id()
            .times(1)
            .returning(|_| Ok(None));

        let uc = GetPaymentUseCase::new(Arc::new(mock_repo));
        let result = uc.execute(payment_id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}
