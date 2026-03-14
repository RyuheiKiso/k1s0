use crate::domain::entity::payment::{Payment, PaymentFilter};
use crate::domain::repository::payment_repository::PaymentRepository;
use std::sync::Arc;

pub struct ListPaymentsUseCase {
    payment_repo: Arc<dyn PaymentRepository>,
}

impl ListPaymentsUseCase {
    pub fn new(payment_repo: Arc<dyn PaymentRepository>) -> Self {
        Self { payment_repo }
    }

    pub async fn execute(&self, filter: &PaymentFilter) -> anyhow::Result<(Vec<Payment>, i64)> {
        let payments = self.payment_repo.find_all(filter).await?;
        let total = self.payment_repo.count(filter).await?;
        Ok((payments, total))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::payment::PaymentStatus;
    use crate::domain::repository::payment_repository::MockPaymentRepository;
    use chrono::Utc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_list_payments_success() {
        let payments = vec![Payment {
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
        }];
        let payments_clone = payments.clone();

        let mut mock_repo = MockPaymentRepository::new();
        mock_repo
            .expect_find_all()
            .times(1)
            .returning(move |_| Ok(payments_clone.clone()));
        mock_repo
            .expect_count()
            .times(1)
            .returning(|_| Ok(1));

        let uc = ListPaymentsUseCase::new(Arc::new(mock_repo));
        let filter = PaymentFilter::default();
        let result = uc.execute(&filter).await;
        assert!(result.is_ok());
        let (found_payments, total) = result.unwrap();
        assert_eq!(found_payments.len(), 1);
        assert_eq!(total, 1);
    }

    #[tokio::test]
    async fn test_list_payments_empty() {
        let mut mock_repo = MockPaymentRepository::new();
        mock_repo
            .expect_find_all()
            .times(1)
            .returning(|_| Ok(vec![]));
        mock_repo
            .expect_count()
            .times(1)
            .returning(|_| Ok(0));

        let uc = ListPaymentsUseCase::new(Arc::new(mock_repo));
        let filter = PaymentFilter::default();
        let result = uc.execute(&filter).await;
        assert!(result.is_ok());
        let (found_payments, total) = result.unwrap();
        assert_eq!(found_payments.len(), 0);
        assert_eq!(total, 0);
    }
}
