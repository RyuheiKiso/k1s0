use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 決済ステータス。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    Initiated,
    Completed,
    Failed,
    Refunded,
}

impl PaymentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Initiated => "initiated",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Refunded => "refunded",
        }
    }

    /// ステータス遷移が有効かどうかを検証する。
    pub fn can_transition_to(&self, next: &Self) -> bool {
        matches!(
            (self, next),
            (Self::Initiated, Self::Completed)
                | (Self::Initiated, Self::Failed)
                | (Self::Completed, Self::Refunded)
        )
    }
}

impl std::str::FromStr for PaymentStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "initiated" => Ok(Self::Initiated),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "refunded" => Ok(Self::Refunded),
            _ => Err(format!("invalid payment status: '{}'", s)),
        }
    }
}

impl std::fmt::Display for PaymentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 決済エンティティ。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: Uuid,
    pub order_id: String,
    pub customer_id: String,
    pub amount: i64,
    pub currency: String,
    pub status: PaymentStatus,
    pub payment_method: Option<String>,
    pub transaction_id: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 決済開始リクエスト。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitiatePayment {
    pub order_id: String,
    pub customer_id: String,
    pub amount: i64,
    pub currency: String,
    pub payment_method: Option<String>,
}

/// 決済一覧フィルター。
#[derive(Debug, Clone, Default)]
pub struct PaymentFilter {
    pub order_id: Option<String>,
    pub customer_id: Option<String>,
    pub status: Option<PaymentStatus>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_status_roundtrip() {
        let statuses = vec![
            PaymentStatus::Initiated,
            PaymentStatus::Completed,
            PaymentStatus::Failed,
            PaymentStatus::Refunded,
        ];
        for status in statuses {
            let s = status.as_str();
            let parsed: PaymentStatus = s.parse().unwrap();
            assert_eq!(parsed, status);
        }
    }

    #[test]
    fn test_payment_status_invalid() {
        let result = "unknown".parse::<PaymentStatus>();
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_transitions() {
        assert!(PaymentStatus::Initiated.can_transition_to(&PaymentStatus::Completed));
        assert!(PaymentStatus::Initiated.can_transition_to(&PaymentStatus::Failed));
        assert!(PaymentStatus::Completed.can_transition_to(&PaymentStatus::Refunded));
    }

    #[test]
    fn test_invalid_transitions() {
        assert!(!PaymentStatus::Initiated.can_transition_to(&PaymentStatus::Refunded));
        assert!(!PaymentStatus::Failed.can_transition_to(&PaymentStatus::Completed));
        assert!(!PaymentStatus::Refunded.can_transition_to(&PaymentStatus::Initiated));
        assert!(!PaymentStatus::Completed.can_transition_to(&PaymentStatus::Initiated));
        assert!(!PaymentStatus::Failed.can_transition_to(&PaymentStatus::Refunded));
    }
}
