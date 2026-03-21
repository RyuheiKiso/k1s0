use crate::domain::entity::payment::{InitiatePayment, Payment};
use crate::domain::error::PaymentError;
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

        // 冪等性チェック: 同一 order_id の決済が既に存在する場合、
        // 金額と通貨が一致するならそれを返す（二重課金防止）。
        // 金額または通貨が異なる場合は冪等性違反としてエラーを返す。
        if let Some(existing) = self.payment_repo.find_by_order_id(&input.order_id).await? {
            // 同一 order_id で異なる金額/通貨のリクエストは冪等性違反として拒否する。
            // これにより、誤ったリトライや不正な重複決済を防止する。
            // 金額または通貨が異なる場合は型付きエラーを返し、呼び出し元で適切な HTTP 409 にマッピングする。
            if existing.amount != input.amount || existing.currency != input.currency {
                return Err(PaymentError::IdempotencyViolation {
                    order_id: input.order_id.clone(),
                }
                .into());
            }
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

        // 冪等性チェック: 同一 order_id の決済が存在しないケース
        mock_repo
            .expect_find_by_order_id()
            .times(1)
            .returning(|_| Ok(None));

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

    // 同一 order_id で異なる金額を持つ既存決済がある場合に冪等性違反エラーを返すことを確認する。
    #[tokio::test]
    async fn test_initiate_payment_idempotency_violation_amount_mismatch() {
        let existing = Payment {
            id: Uuid::new_v4(),
            order_id: "ORD-002".to_string(),
            customer_id: "CUST-001".to_string(),
            amount: 3000, // 既存は 3000
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
        let existing_clone = existing.clone();

        let mut mock_repo = MockPaymentRepository::new();
        // find_by_order_id は金額が異なる既存決済を返す
        mock_repo
            .expect_find_by_order_id()
            .times(1)
            .returning(move |_| Ok(Some(existing_clone.clone())));

        let uc = InitiatePaymentUseCase::new(Arc::new(mock_repo));
        let input = InitiatePayment {
            order_id: "ORD-002".to_string(),
            customer_id: "CUST-001".to_string(),
            amount: 5000, // 新規リクエストは 5000（不一致）
            currency: "JPY".to_string(),
            payment_method: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("冪等性違反"));
    }

    // 同一 order_id で異なる通貨を持つ既存決済がある場合に冪等性違反エラーを返すことを確認する。
    #[tokio::test]
    async fn test_initiate_payment_idempotency_violation_currency_mismatch() {
        let existing = Payment {
            id: Uuid::new_v4(),
            order_id: "ORD-003".to_string(),
            customer_id: "CUST-001".to_string(),
            amount: 5000,
            currency: "USD".to_string(), // 既存は USD
            status: PaymentStatus::Initiated,
            payment_method: Some("credit_card".to_string()),
            transaction_id: None,
            error_code: None,
            error_message: None,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let existing_clone = existing.clone();

        let mut mock_repo = MockPaymentRepository::new();
        mock_repo
            .expect_find_by_order_id()
            .times(1)
            .returning(move |_| Ok(Some(existing_clone.clone())));

        let uc = InitiatePaymentUseCase::new(Arc::new(mock_repo));
        let input = InitiatePayment {
            order_id: "ORD-003".to_string(),
            customer_id: "CUST-001".to_string(),
            amount: 5000,
            currency: "JPY".to_string(), // 新規は JPY（不一致）
            payment_method: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("冪等性違反"));
    }

    // 同一 order_id・同一金額・同一通貨の場合は既存決済を返す（正常な冪等性）ことを確認する。
    #[tokio::test]
    async fn test_initiate_payment_idempotency_same_amount_and_currency() {
        let existing = sample_payment();
        let existing_clone = existing.clone();

        let mut mock_repo = MockPaymentRepository::new();
        mock_repo
            .expect_find_by_order_id()
            .times(1)
            .returning(move |_| Ok(Some(existing_clone.clone())));

        let uc = InitiatePaymentUseCase::new(Arc::new(mock_repo));
        let input = sample_initiate_payment(); // amount=5000, currency="JPY" で一致
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, existing.id);
    }
}
