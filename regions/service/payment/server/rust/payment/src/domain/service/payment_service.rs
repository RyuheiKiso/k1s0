use crate::domain::entity::payment::{InitiatePayment, PaymentStatus};
use crate::domain::error::PaymentError;

/// 決済ドメインサービス — ドメインルールのバリデーションを担当。
pub struct PaymentDomainService;

impl PaymentDomainService {
    /// 決済開始入力を検証する。
    /// amount の上限チェックと currency の ISO 4217 フォーマット検証を含む。
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
        // amount 上限チェック: i64::MAX / 100 を超える金額は現実的でなく、
        // 数値オーバーフローや UI 表示の破綻を防ぐため上限を設ける。
        const MAX_AMOUNT: i64 = i64::MAX / 100;
        if input.amount > MAX_AMOUNT {
            return Err(PaymentError::ValidationFailed(format!(
                "amount must not exceed {} (got {})",
                MAX_AMOUNT, input.amount
            )));
        }
        if input.currency.trim().is_empty() {
            return Err(PaymentError::ValidationFailed(
                "currency must not be empty".to_string(),
            ));
        }
        // currency フォーマット検証: ISO 4217 に準拠した3文字の大文字アルファベットを要求する。
        // 例: "JPY", "USD", "EUR"
        let currency = input.currency.trim();
        if currency.len() != 3 || !currency.chars().all(|c| c.is_ascii_uppercase()) {
            return Err(PaymentError::ValidationFailed(format!(
                "currency must be a 3-letter uppercase ISO 4217 code (got '{}')",
                currency
            )));
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

    // amount が上限（i64::MAX / 100）を超える場合にエラーを返すことを確認する。
    #[test]
    fn test_validate_initiate_payment_amount_exceeds_max() {
        let mut input = valid_initiate_payment();
        input.amount = i64::MAX / 100 + 1;
        let result = PaymentDomainService::validate_initiate_payment(&input);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("amount"));
    }

    // amount が上限（i64::MAX / 100）ちょうどの場合は成功することを確認する。
    #[test]
    fn test_validate_initiate_payment_amount_at_max_boundary() {
        let mut input = valid_initiate_payment();
        input.amount = i64::MAX / 100;
        assert!(PaymentDomainService::validate_initiate_payment(&input).is_ok());
    }

    // currency が3文字の大文字アルファベット以外（小文字）の場合にエラーを返すことを確認する。
    #[test]
    fn test_validate_initiate_payment_currency_lowercase() {
        let mut input = valid_initiate_payment();
        input.currency = "jpy".to_string();
        let result = PaymentDomainService::validate_initiate_payment(&input);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("currency"));
    }

    // currency が3文字より長い場合にエラーを返すことを確認する。
    #[test]
    fn test_validate_initiate_payment_currency_too_long() {
        let mut input = valid_initiate_payment();
        input.currency = "JPYY".to_string();
        let result = PaymentDomainService::validate_initiate_payment(&input);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("currency"));
    }

    // currency が3文字より短い場合にエラーを返すことを確認する。
    #[test]
    fn test_validate_initiate_payment_currency_too_short() {
        let mut input = valid_initiate_payment();
        input.currency = "JP".to_string();
        let result = PaymentDomainService::validate_initiate_payment(&input);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("currency"));
    }

    // currency が数字を含む場合にエラーを返すことを確認する。
    #[test]
    fn test_validate_initiate_payment_currency_with_digit() {
        let mut input = valid_initiate_payment();
        input.currency = "JP1".to_string();
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
