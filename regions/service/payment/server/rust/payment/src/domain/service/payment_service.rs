use crate::domain::entity::payment::{InitiatePayment, PaymentStatus};
use crate::domain::error::PaymentError;

/// 決済ドメインサービス — ドメインルールのバリデーションを担当。
pub struct PaymentDomainService;

impl PaymentDomainService {
    /// 決済開始入力を検証する。
    pub fn validate_initiate_payment(input: &InitiatePayment) -> Result<(), PaymentError> {
        if input.order_id.trim().is_empty() {
            return Err(PaymentError::ValidationFailed(
                "order_id must not be empty".to_string(),
            ));
        }
        if input.customer_id.trim().is_empty() {
            return Err(PaymentError::ValidationFailed(
                "customer_id must not be empty".to_string(),
            ));
        }
        if input.amount <= 0 {
            return Err(PaymentError::ValidationFailed(
                "amount must be greater than zero".to_string(),
            ));
        }
        if input.currency.trim().is_empty() {
            return Err(PaymentError::ValidationFailed(
                "currency must not be empty".to_string(),
            ));
        }
        Ok(())
    }

    /// ステータス遷移を検証する。
    pub fn validate_status_transition(
        current: &PaymentStatus,
        next: &PaymentStatus,
    ) -> Result<(), PaymentError> {
        if !current.can_transition_to(next) {
            return Err(PaymentError::InvalidStatusTransition {
                from: current.to_string(),
                to: next.to_string(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_initiate_payment() -> InitiatePayment {
        InitiatePayment {
            order_id: "ORD-001".to_string(),
            customer_id: "CUST-001".to_string(),
            amount: 5000,
            currency: "JPY".to_string(),
            payment_method: Some("credit_card".to_string()),
        }
    }

    #[test]
    fn test_validate_initiate_payment_success() {
        let input = valid_initiate_payment();
        assert!(PaymentDomainService::validate_initiate_payment(&input).is_ok());
    }

    #[test]
    fn test_validate_initiate_payment_empty_order_id() {
        let mut input = valid_initiate_payment();
        input.order_id = "".to_string();
        let result = PaymentDomainService::validate_initiate_payment(&input);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("order_id"));
    }

    #[test]
    fn test_validate_initiate_payment_empty_customer_id() {
        let mut input = valid_initiate_payment();
        input.customer_id = "".to_string();
        let result = PaymentDomainService::validate_initiate_payment(&input);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("customer_id"));
    }

    #[test]
    fn test_validate_initiate_payment_zero_amount() {
        let mut input = valid_initiate_payment();
        input.amount = 0;
        let result = PaymentDomainService::validate_initiate_payment(&input);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("amount"));
    }

    #[test]
    fn test_validate_initiate_payment_negative_amount() {
        let mut input = valid_initiate_payment();
        input.amount = -100;
        let result = PaymentDomainService::validate_initiate_payment(&input);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("amount"));
    }

    #[test]
    fn test_validate_initiate_payment_empty_currency() {
        let mut input = valid_initiate_payment();
        input.currency = "".to_string();
        let result = PaymentDomainService::validate_initiate_payment(&input);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("currency"));
    }

    #[test]
    fn test_valid_status_transition() {
        assert!(PaymentDomainService::validate_status_transition(
            &PaymentStatus::Initiated,
            &PaymentStatus::Completed,
        )
        .is_ok());
    }

    #[test]
    fn test_invalid_status_transition() {
        let result = PaymentDomainService::validate_status_transition(
            &PaymentStatus::Failed,
            &PaymentStatus::Completed,
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("invalid status transition"));
    }
}
